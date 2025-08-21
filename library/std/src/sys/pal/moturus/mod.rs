#![allow(unsafe_op_in_unsafe_fn)]

pub mod fd;
pub mod fs;
pub mod io;
pub mod io_pipe;
pub mod net;
pub mod os;
pub mod stdio;
pub mod thread;
pub mod time;

pub use moto_rt::futex;

pub mod pipe {
    pub use super::io_pipe::{IoPipe as AnonPipe, read2};
    use crate::io;

    #[inline]
    pub fn pipe() -> io::Result<(AnonPipe, AnonPipe)> {
        Err(io::Error::UNSUPPORTED_PLATFORM)
    }
}

mod common;
pub use common::*;

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn moturus_start() -> ! {
    // Initialize the runtime.
    moto_rt::start();

    // Call main.
    unsafe extern "C" {
        fn main(_: isize, _: *const *const u8, _: u8) -> i32;
    }
    let result = unsafe { main(0, core::ptr::null(), 0) };

    // Terminate the process.
    moto_rt::process::exit(result)
}

// This function is needed by the panic runtime. The symbol is named in
// pre-link args for the target specification, so keep that in sync.
#[cfg(not(test))]
#[unsafe(no_mangle)]
// NB. used by both libunwind and libpanic_abort
pub extern "C" fn __rust_abort() {
    moto_rt::process::exit(-1)
}

pub(crate) use crate::os::moturus::map_moturus_error;

pub fn is_interrupted(_code: i32) -> bool {
    false
}
