#![unstable(feature = "moturus_ext", issue = "none")]

use crate::sealed::Sealed;
use crate::sys_common::AsInner;

#[unstable(feature = "moturus_ext", issue = "none")]
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
        E_INVALID_ARGUMENT => ErrorKind::InvalidInput,
        E_NOT_READY => ErrorKind::WouldBlock,
        E_NOT_CONNECTED => ErrorKind::NotConnected,
        _ => ErrorKind::Other,
    };

    crate::io::Error::from(kind)
}

#[unstable(feature = "moturus_ext", issue = "none")]
pub trait ChildExt: Sealed {
    /// Extracts the main thread raw handle, without taking ownership
    #[unstable(feature = "moturus_ext", issue = "none")]
    fn sys_handle(&self) -> u64;
}

#[unstable(feature = "moturus_ext", issue = "none")]
impl ChildExt for crate::process::Child {
    fn sys_handle(&self) -> u64 {
        self.as_inner().handle()
    }
}
