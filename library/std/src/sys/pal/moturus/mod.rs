#![allow(unsafe_op_in_unsafe_fn)]

pub mod alloc;
pub mod args;
pub mod env;
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
    moto_runtime::moturus_start_rt();
    extern "C" {
        fn main(_: isize, _: *const *const u8, _: u8) -> i32;
    }
    let result = unsafe { main(0, core::ptr::null(), 0) };
    moto_runtime::sys_exit(result as u64);
}

// This function is needed by the panic runtime. The symbol is named in
// pre-link args for the target specification, so keep that in sync.
#[cfg(not(test))]
#[no_mangle]
// NB. used by both libunwind and libpanic_abort
pub extern "C" fn __rust_abort() {
    moto_runtime::sys_exit(u64::MAX)
}

pub fn map_moturus_error(err: moto_rt::ErrorCode) -> crate::io::Error {
    use moto_rt::error::*;

    use crate::io::ErrorKind;

    let kind: ErrorKind = match err {
        E_ALREADY_IN_USE => ErrorKind::AlreadyExists,
        E_INVALID_FILENAME => ErrorKind::InvalidFilename,
        E_NOT_FOUND => ErrorKind::NotFound,
        E_TIMED_OUT => ErrorKind::TimedOut,
        E_NOT_IMPLEMENTED => ErrorKind::Unsupported,
        E_FILE_TOO_LARGE => ErrorKind::FileTooLarge,
        E_UNEXPECTED_EOF => ErrorKind::UnexpectedEof,
        _ => ErrorKind::Other,
    };

    crate::io::Error::from(kind)
}

pub fn is_interrupted(_code: i32) -> bool {
    false
}
