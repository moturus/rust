#![allow(unsafe_op_in_unsafe_fn)]

pub mod alloc;
pub mod args;
pub mod env;
pub mod fs;
pub mod io;
pub mod net;
pub mod os;
pub mod pipe;
pub mod process;
pub mod stdio;
pub mod thread;
#[cfg(target_thread_local)]
pub mod thread_local_dtor;
pub mod thread_local_key;
pub mod time;

pub use moto_runtime::futex;

mod common;
pub use common::*;

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn moturus_start() -> ! {
    moto_runtime::moturus_start_rt();
    extern "C" { fn main(_: isize, _: *const *const u8, _: u8) -> i32; }
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

pub fn map_moturus_error(err: moto_runtime::ErrorCode) -> crate::io::Error {
    use crate::io::ErrorKind;
    use moto_runtime::ErrorCode;

    let kind: ErrorKind = match err {
        ErrorCode::AlreadyInUse => ErrorKind::AlreadyExists,
        ErrorCode::InvalidFilename => ErrorKind::InvalidFilename,
        ErrorCode::NotFound => ErrorKind::NotFound,
        ErrorCode::TimedOut => ErrorKind::TimedOut,
        ErrorCode::NotImplemented => ErrorKind::Unsupported,
        ErrorCode::FileTooLarge => ErrorKind::FileTooLarge,
        ErrorCode::UnexpectedEof => ErrorKind::UnexpectedEof,
        _ => ErrorKind::Other
    };

    crate::io::Error::from(kind)
}

pub fn is_interrupted(_code: i32) -> bool {
    false
}
