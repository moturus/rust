#![unstable(reason = "not public", issue = "none", feature = "fd")]
// use crate::cmp;
// use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut, Read};
use crate::io::{self, BorrowedCursor, IoSliceMut, Read};
use crate::os::fd::RawFd;
use crate::os::fd::{AsFd, AsRawFd, BorrowedFd};
use crate::os::fd::FromRawFd;
use crate::os::fd::IntoRawFd;
use crate::os::fd::OwnedFd;
use crate::sys_common::{AsInner, FromInner, IntoInner};
use super::map_moturus_error;

#[derive(Debug)]
pub struct FileDesc(OwnedFd);

impl FileDesc {
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        moto_rt::fs::read(self.as_raw_fd(), buf).map_err(map_moturus_error)
    }

    pub fn read_buf(&self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        crate::io::default_read_buf(|buf| self.read(buf), cursor)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        io::default_read_vectored(|b| self.read(b), bufs)
    }

    #[inline]
    pub fn is_read_vectored(&self) -> bool {
        false
    }

    pub fn set_nonblocking(&self, _nonblocking: bool) -> io::Result<()> {
        todo!()
    }

    #[inline]
    pub fn duplicate(&self) -> io::Result<FileDesc> {
        // Ok(Self(self.0.try_clone()?))
        todo!()
    }
}

impl<'a> Read for &'a FileDesc {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf(cursor)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (**self).read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        (**self).is_read_vectored()
    }
}

impl AsInner<OwnedFd> for FileDesc {
    #[inline]
    fn as_inner(&self) -> &OwnedFd {
        &self.0
    }
}

impl IntoInner<OwnedFd> for FileDesc {
    fn into_inner(self) -> OwnedFd {
        self.0
    }
}

impl FromInner<OwnedFd> for FileDesc {
    fn from_inner(owned_fd: OwnedFd) -> Self {
        Self(owned_fd)
    }
}

impl AsFd for FileDesc {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for FileDesc {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl IntoRawFd for FileDesc {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl FromRawFd for FileDesc {
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self {
        Self(FromRawFd::from_raw_fd(raw_fd))
    }
}
