/// This macros allows to hide panicing messages in output binary when feature `no-msgs` is present.
macro_rules! expect {
    ($val:expr, $msg:expr) => {
        if cfg!(feature = "no-msgs") {
            $val.unwrap()
        } else {
            $val.expect($msg)
        }
    };
}

macro_rules! panic_msg {
    ($($t:tt)*) => {
        if cfg!(feature = "no-msgs") {
            unimplemented!()
        } else {
            panic!($($t)*)
        }
    };
}

/// Creates zero terminated string.
macro_rules! pc_str {
    ($cstr:expr) => {
        windows::core::PCSTR(concat!($cstr, "\x00").as_ptr() as _)
    };
}

#[cfg(all(feature = "parking-lot", feature = "spin-lock"))]
compile_error!("Only one of features `parking-lot`, `spin-lock` must be enabled.");

#[cfg(not(any(feature = "parking-lot", feature = "spin-lock")))]
compile_error!("One of the features `parking-lot`, `spin-lock` must be enabled.");

mod app;
pub use app::OpenGLApp;

mod input;
mod painter;
mod shader;
mod utils;