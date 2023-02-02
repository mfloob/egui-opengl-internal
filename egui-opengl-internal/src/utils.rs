use std::ffi::{CString};
use winapi::um::{
    libloaderapi::{GetModuleHandleA, GetProcAddress},
    consoleapi::AllocConsole,
};
use winapi::um::wingdi::{wglGetProcAddress};

pub unsafe fn get_proc_address(function_name: &str) -> *const usize {
    let o = CString::new("opengl32.dll").unwrap();
    let opengl32 = GetModuleHandleA(o.as_ptr());           
    let c = CString::new(function_name).unwrap();
    let process_address = GetProcAddress(opengl32, c.as_ptr());

    if process_address as isize > 0 {
        return process_address as *const usize;
    }

    let c_proc_name = CString::new(function_name).unwrap();
    let process_address = wglGetProcAddress(c_proc_name.as_ptr());
    process_address as *const usize
}

pub fn alloc_console() {
    unsafe {
        AllocConsole();
    }
}