use std::ffi::CString;

use windows::{
    core::PCSTR,
    Win32::{
        Foundation::HINSTANCE,
        Graphics::OpenGL::wglGetProcAddress,
        System::{
            Console::{AllocConsole, FreeConsole},
            LibraryLoader::{FreeLibraryAndExitThread, GetModuleHandleA, GetProcAddress},
        },
    },
};

pub unsafe fn get_proc_address(function_name: &str) -> *const usize {
    let opengl32 = get_module("opengl32.dll");
    let c = CString::new(function_name).unwrap();
    let process_address = GetProcAddress(opengl32, PCSTR::from_raw(c.as_ptr() as *const u8));

    if let Some(process_address) = process_address {
        return process_address as _;
    }

    let c_proc_name = CString::new(function_name).unwrap();
    let process_address = wglGetProcAddress(PCSTR::from_raw(c_proc_name.as_ptr() as *const u8));

    if let Some(process_address) = process_address {
        return process_address as _;
    }

    // this shouldn't silently error tbh, but im going to copy the old behavior
    0 as *const usize
}

pub fn get_module(module_name: &str) -> HINSTANCE {
    unsafe {
        let o = CString::new(module_name).unwrap();
        let module = GetModuleHandleA(PCSTR::from_raw(o.as_ptr() as *const u8));

        if let Ok(module) = module {
            module
        } else {
            // this also shouldn't silently error
            HINSTANCE(0)
        }
    }
}

pub fn alloc_console() {
    unsafe {
        AllocConsole();
    }
}

pub fn free_console() {
    unsafe {
        FreeConsole();
    }
}

pub fn unload() {
    unsafe {
        let module = get_module("example_wnd.dll");
        FreeLibraryAndExitThread(module, 0);
    }
}
