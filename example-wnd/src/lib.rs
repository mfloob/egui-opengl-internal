use egui::{Color32, Context, Key, Modifiers, RichText, ScrollArea, Slider, Widget};
use egui_opengl_internal::{utils, OpenGLApp};
use retour::static_detour;
use std::{intrinsics::transmute, sync::Once};
use windows::{
    core::HRESULT,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{WindowFromDC, HDC},
        UI::WindowsAndMessaging::{CallWindowProcW, SetWindowLongPtrA, GWLP_WNDPROC, WNDPROC},
    },
};

#[no_mangle]
extern "stdcall" fn DllMain(hinst: usize, reason: u32) -> i32 {
    if reason == 1 {
        std::thread::spawn(move || unsafe { main_thread(hinst) });
    }

    if reason == 0 {
        unsafe {
            WglSwapBuffersHook.disable().unwrap();
            let wnd_proc = OLD_WND_PROC.unwrap().unwrap();
            let _: Option<WNDPROC> = Some(transmute(SetWindowLongPtrA(
                APP.get_window(),
                GWLP_WNDPROC,
                wnd_proc as usize as _,
            )));

            utils::free_console();
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    1
}

static mut APP: OpenGLApp<i32> = OpenGLApp::new();
static mut OLD_WND_PROC: Option<WNDPROC> = None;
static mut EXITING: bool = false;

type FnWglSwapBuffers = unsafe extern "stdcall" fn(HDC) -> HRESULT;
static_detour! {
    static WglSwapBuffersHook: unsafe extern "stdcall" fn(HDC) -> HRESULT;
}

fn hk_wgl_swap_buffers(hdc: HDC) -> HRESULT {
    unsafe {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            println!("wglSwapBuffers successfully hooked.");

            let window = WindowFromDC(hdc);
            APP.init_default(hdc, window, ui);

            OLD_WND_PROC = Some(transmute(SetWindowLongPtrA(
                window,
                GWLP_WNDPROC,
                hk_wnd_proc as usize as _,
            )));
        });

        APP.render(hdc);
        WglSwapBuffersHook.call(hdc)
    }
}

unsafe extern "stdcall" fn hk_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        println!("CallWindowProcW successfully hooked.");
    });

    let egui_wants_input = APP.wnd_proc(msg, wparam, lparam);
    if egui_wants_input {
        return LRESULT(1);
    }

    CallWindowProcW(OLD_WND_PROC.unwrap(), hwnd, msg, wparam, lparam)
}

fn ui(ctx: &Context, _: &mut i32) {
    unsafe {
        egui::containers::Window::new("Main menu").show(ctx, |ui| {
            test_ui(ctx, ui);

            ui.separator();
            if ui.button("exit").clicked() {
                EXITING = true;
            }
        });
    }
}

unsafe fn main_thread(_hinst: usize) {
    utils::alloc_console();

    let wgl_swap_buffers = utils::get_proc_address("wglSwapBuffers");
    let fn_wgl_swap_buffers: FnWglSwapBuffers = std::mem::transmute(wgl_swap_buffers);

    println!("wglSwapBuffers: {:X}", wgl_swap_buffers as usize);

    WglSwapBuffersHook
        .initialize(fn_wgl_swap_buffers, hk_wgl_swap_buffers)
        .unwrap()
        .enable()
        .unwrap();

    #[allow(clippy::empty_loop)]
    while !EXITING {}
    utils::unload();
}

unsafe fn test_ui(ctx: &egui::Context, ui: &mut egui::Ui) {
    // You should not use statics like this, it's made
    // this way for the sake of example.
    static mut UI_CHECK: bool = true;
    static mut TEXT: Option<String> = None;
    static mut VALUE: f32 = 0.;
    static mut COLOR: [f32; 3] = [0., 0., 0.];
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {});

    if TEXT.is_none() {
        TEXT = Some(String::from("Test"));
    }
    ui.label(RichText::new("Test").color(Color32::LIGHT_BLUE));
    ui.label(RichText::new("Other").color(Color32::WHITE));
    ui.separator();

    let input = ctx.input(|input| input.pointer.clone());
    ui.label(format!(
        "X1: {} X2: {}",
        input.button_down(egui::PointerButton::Extra1),
        input.button_down(egui::PointerButton::Extra2)
    ));

    let mods = ui.input(|input| input.modifiers);
    ui.label(format!(
        "Ctrl: {} Shift: {} Alt: {}",
        mods.ctrl, mods.shift, mods.alt
    ));

    if ui.input(|input| input.modifiers.matches(Modifiers::CTRL) && input.key_pressed(Key::R)) {
        println!("Pressed");
    }

    ui.checkbox(&mut UI_CHECK, "Some checkbox");
    ui.text_edit_singleline(TEXT.as_mut().unwrap());
    ScrollArea::vertical().max_height(200.).show(ui, |ui| {
        for i in 1..=100 {
            ui.label(format!("Label: {}", i));
        }
    });

    Slider::new(&mut VALUE, -1.0..=1.0).ui(ui);

    ui.color_edit_button_rgb(&mut COLOR);

    ui.label(format!(
        "{:?}",
        &ui.input(|input| input.pointer.button_down(egui::PointerButton::Primary))
    ));
}
