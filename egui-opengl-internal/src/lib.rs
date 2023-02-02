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

#[cfg(all(feature = "parking-lot", feature = "spin-lock"))]
compile_error!("Only one of features `parking-lot`, `spin-lock` must be enabled.");

#[cfg(not(any(feature = "parking-lot", feature = "spin-lock")))]
compile_error!("One of the features `parking-lot`, `spin-lock` must be enabled.");

mod app;
pub use app::OpenGLApp;

mod input;
mod painter;
mod shader;
pub mod utils;