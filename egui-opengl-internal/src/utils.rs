use std::ffi::{CString};
use winapi::{
    um::{
        libloaderapi::{GetModuleHandleA, GetProcAddress, FreeLibraryAndExitThread},
        consoleapi::{AllocConsole},
        wincon::{FreeConsole},
        wingdi::{wglGetProcAddress},
    }, 
    shared::minwindef::HINSTANCE__,
};

pub unsafe fn get_proc_address(function_name: &str) -> *const usize {
    let opengl32 = get_module("opengl32.dll");           
    let c = CString::new(function_name).unwrap();
    let process_address = GetProcAddress(opengl32, c.as_ptr());

    if process_address as isize > 0 {
        return process_address as *const usize;
    }

    let c_proc_name = CString::new(function_name).unwrap();
    let process_address = wglGetProcAddress(c_proc_name.as_ptr());
    process_address as *const usize
}

pub fn get_module(module_name: &str) -> *mut HINSTANCE__ {
    unsafe {
        let o = CString::new(module_name).unwrap();
        GetModuleHandleA(o.as_ptr())
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