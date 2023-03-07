use crate::{input::InputCollector, painter, utils};
use clipboard::{windows_clipboard::WindowsClipboardContext, ClipboardProvider};
use egui::Context;
use once_cell::sync::OnceCell;
use std::ops::DerefMut;
use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT, WPARAM},
    Graphics::{
        Gdi::HDC,
        OpenGL::{wglCreateContext, wglGetCurrentContext, wglMakeCurrent, HGLRC},
    },
    UI::WindowsAndMessaging::{GetClientRect, WM_SIZING},
};

#[allow(clippy::type_complexity)]
struct AppData<T> {
    ui: Box<dyn FnMut(&Context, &mut T) + 'static>,
    gl_context: HGLRC,
    window: HWND,
    painter: painter::Painter,
    input_collector: InputCollector,
    ctx: Context,
    client_rect: (u32, u32),
    state: T,
}

#[cfg(feature = "parking-lot")]
use parking_lot::{Mutex, MutexGuard};
#[cfg(feature = "spin-lock")]
use spin::lock_api::{Mutex, MutexGuard};

use lock_api::MappedMutexGuard;

/// Heart and soul of this integration.
/// Main methods you are going to use are:
/// * [`Self::render`] - Should be called inside of wglSwapBuffers hook.
/// * [`Self::wnd_proc`] - Should be called on each `WndProc`.
pub struct OpenGLApp<T = ()> {
    data: Mutex<Option<AppData<T>>>,
    hwnd: OnceCell<HWND>,
}

impl<T> OpenGLApp<T> {
    /// Creates new [`OpenGLApp`] in const context. You are supposed to create a single static item to store the application state.
    pub const fn new() -> Self {
        Self {
            data: Mutex::new(None),
            hwnd: OnceCell::new(),
        }
    }

    /// Checks if the app is ready to draw and if it's safe to invoke `render`, `wndproc`, etc.
    /// `true` means that you have already called an `init_*` on the application.
    pub fn is_ready(&self) -> bool {
        self.hwnd.get().is_some()
    }

    /// Initializes application and state. You should call this only once!
    pub fn init_with_state_context(
        &self,
        hdc: HDC,
        window: HWND,
        ui: impl FnMut(&Context, &mut T) + 'static,
        state: T,
        context: Context,
    ) {
        unsafe {
            if self.hwnd.get().is_some() {
                panic_msg!("You must call init only once");
            }

            if window.0 == -1 {
                panic_msg!("Invalid output window descriptor");
            }

            let _ = self.hwnd.set(window);

            // loads gl with all the opengl functions using get_proc_address which is hardcoded to look in the opengl32.dll module
            gl::load_with(|s| utils::get_proc_address(s) as *const _);

            let o_context = wglGetCurrentContext();
            let gl_context = wglCreateContext(hdc).unwrap();
            wglMakeCurrent(hdc, gl_context).unwrap();

            let painter = painter::Painter::new();

            *self.data.lock() = Some(AppData {
                input_collector: InputCollector::new(window),
                ui: Box::new(ui),
                gl_context,
                window,
                ctx: context,
                client_rect: (0, 0),
                state,
                painter,
            });

            wglMakeCurrent(hdc, o_context).unwrap();
        }
    }

    /// Initializes application and state. Sets egui's context to default value. You should call this only once!
    #[inline]
    pub fn init_with_state(
        &self,
        hdc: HDC,
        window: HWND,
        ui: impl FnMut(&Context, &mut T) + 'static,
        state: T,
    ) {
        self.init_with_state_context(hdc, window, ui, state, Context::default())
    }

    /// Initializes application and state while allowing you to mutate the initial state of the egui's context. You should call this only once!
    #[inline]
    pub fn init_with_mutate(
        &self,
        hdc: HDC,
        window: HWND,
        ui: impl FnMut(&Context, &mut T) + 'static,
        mut state: T,
        mutate: impl FnOnce(&mut Context, &mut T),
    ) {
        let mut ctx = Context::default();
        mutate(&mut ctx, &mut state);

        self.init_with_state_context(hdc, window, ui, state, ctx);
    }

    #[cfg(feature = "parking-lot")]
    pub fn lock_state(&self) -> MappedMutexGuard<'_, parking_lot::RawMutex, T> {
        MutexGuard::map(self.data.lock(), |app| &mut app.as_mut().unwrap().state)
    }

    #[cfg(feature = "spin-lock")]
    pub fn lock_state(&self) -> MappedMutexGuard<'_, spin::mutex::Mutex<()>, T> {
        MutexGuard::map(self.data.lock(), |app| &mut app.as_mut().unwrap().state)
    }

    fn lock_data(&self) -> impl DerefMut<Target = AppData<T>> + '_ {
        MutexGuard::map(self.data.lock(), |app| {
            expect!(app.as_mut(), "You need to call init first")
        })
    }
}

impl<T: Default> OpenGLApp<T> {
    /// Initializes application and sets the state to its default value. You should call this only once!
    #[inline]
    pub fn init_default(&self, hdc: HDC, window: HWND, ui: impl FnMut(&Context, &mut T) + 'static) {
        self.init_with_state_context(hdc, window, ui, T::default(), Context::default());
    }
}

impl<T> OpenGLApp<T> {
    /// Present call. Should be called once per original present call, before or inside of hook.
    #[allow(clippy::cast_ref_to_mut)]
    pub fn render(&self, hdc: HDC) {
        unsafe {
            let this = &mut *self.lock_data();

            let o_context = wglGetCurrentContext();
            wglMakeCurrent(hdc, this.gl_context).unwrap();

            let output = this.ctx.run(this.input_collector.collect_input(), |ctx| {
                (this.ui)(ctx, &mut this.state);
            });

            if !output.platform_output.copied_text.is_empty() {
                let _ = WindowsClipboardContext.set_contents(output.platform_output.copied_text);
            }

            if output.shapes.is_empty() {
                wglMakeCurrent(hdc, o_context).unwrap();
                return;
            }

            let client_rect = self.poll_client_rect(this);
            let clipped_shapes = this.ctx.tessellate(output.shapes);
            this.painter.paint_and_update_textures(
                1.0,
                &clipped_shapes,
                &output.textures_delta,
                &client_rect,
            );

            wglMakeCurrent(hdc, o_context).unwrap();
        }
    }

    /// Call on each `WndProc` occurence.
    /// Returns `true` if message was recognized and dispatched by input handler,
    /// `false` otherwise.
    #[inline]
    pub fn wnd_proc(&self, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> bool {
        let this = &mut *self.lock_data();
        this.input_collector.process(umsg, wparam.0, lparam.0);

        if umsg == WM_SIZING {
            this.client_rect = self.get_client_rect();
        }

        let egui_input = this.ctx.wants_keyboard_input() || this.ctx.wants_pointer_input();
        egui_input
    }

    pub fn get_window(&self) -> HWND {
        let data = &mut *self.lock_data();
        data.window
    }
}

impl<T> OpenGLApp<T> {
    #[inline]
    fn poll_client_rect(&self, data: &mut AppData<T>) -> (u32, u32) {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| {
            data.client_rect = self.get_client_rect();
        });

        data.client_rect
    }

    #[inline]
    fn get_client_rect(&self) -> (u32, u32) {
        let mut rect = RECT::default();
        unsafe {
            GetClientRect(
                *expect!(self.hwnd.get(), "You need to call init first"),
                &mut rect,
            );
        }

        (
            (rect.right - rect.left) as u32,
            (rect.bottom - rect.top) as u32,
        )
    }
}
