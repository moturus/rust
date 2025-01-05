#![allow(unsafe_op_in_unsafe_fn)]

pub mod alloc;
pub mod args;
pub mod env;
pub mod fd;
pub mod fs;
pub mod io;
mod io_pipe;
pub mod net;
pub mod os;
pub mod process;
pub mod stdio;
pub mod thread;
pub mod time;

pub use moto_rt::futex;

pub mod pipe {
    pub use super::io_pipe::{read2, IoPipe as AnonPipe};
    use crate::io;
    use crate::pipe::{PipeReader, PipeWriter};
    use crate::process::Stdio;

    #[inline]
    pub fn pipe() -> io::Result<(AnonPipe, AnonPipe)> {
        Err(io::Error::UNSUPPORTED_PLATFORM)
    }

    #[unstable(feature = "anonymous_pipe", issue = "127154")]
    impl From<PipeReader> for Stdio {
        fn from(_pipe: PipeReader) -> Self {
            todo!()
        }
    }

    #[unstable(feature = "anonymous_pipe", issue = "127154")]
    impl From<PipeWriter> for Stdio {
        fn from(_pipe: PipeWriter) -> Self {
            todo!()
        }
    }
}

mod common;
pub use common::*;

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn moturus_start() -> ! {
    // Initialize the runtime.
    moto_rt::start();

    // Call main.
    extern "C" {
        fn main(_: isize, _: *const *const u8, _: u8) -> i32;
    }
    let result = unsafe { main(0, core::ptr::null(), 0) };

    // Terminate the process.
    moto_rt::process::exit(result)
}

// This function is needed by the panic runtime. The symbol is named in
// pre-link args for the target specification, so keep that in sync.
#[cfg(not(test))]
#[no_mangle]
// NB. used by both libunwind and libpanic_abort
pub extern "C" fn __rust_abort() {
    moto_rt::process::exit(-1)
}

pub(crate) use crate::os::moturus::map_moturus_error;

pub fn is_interrupted(_code: i32) -> bool {
    false
}
