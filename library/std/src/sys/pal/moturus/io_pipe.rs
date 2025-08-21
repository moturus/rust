use super::map_moturus_error;
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut};
use crate::os::fd::RawFd;
use crate::os::fd::{AsFd, AsRawFd, FromRawFd, IntoRawFd, BorrowedFd, OwnedFd};
use crate::sys::fd::FileDesc;
use crate::sys_common::{/* AsInner, AsInnerMut, */ FromInner, IntoInner};

#[derive(Debug)]
pub struct IoPipe(FileDesc);

impl From<moto_rt::RtFd> for IoPipe {
    fn from(rt_fd: moto_rt::RtFd) -> IoPipe {
        unsafe { IoPipe::from_raw_fd(rt_fd) }
    }
}

impl IoPipe {
    pub fn try_clone(&self) -> io::Result<Self> {
        todo!()
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        moto_rt::fs::read(self.as_raw_fd(), buf).map_err(map_moturus_error)
    }

    pub fn read_buf(&self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        crate::io::default_read_buf(|buf| self.read(buf), cursor)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        crate::io::default_read_vectored(|b| self.read(b), bufs)
    }

    pub fn is_read_vectored(&self) -> bool {
        false
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        moto_rt::fs::write(self.as_raw_fd(), buf).map_err(map_moturus_error)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        crate::io::default_write_vectored(|b| self.write(b), bufs)
    }

    pub fn is_write_vectored(&self) -> bool {
        false
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut temp_vec = alloc::vec::Vec::new();
        let mut size = 0_usize;
        loop {
            temp_vec.resize(256, 0_u8);
            match self.read(&mut temp_vec[..]) {
                Ok(sz) => {
                    if sz == 0 {
                        return Ok(size);
                    }
                    size += sz;
                    temp_vec.truncate(sz);
                    buf.append(&mut temp_vec);
                }
                Err(err) => {
                    if size != 0 {
                        return Ok(size);
                    } else {
                        return Err(err);
                    }
                }
            }
        }
    }
}

impl AsRawFd for IoPipe {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl FromRawFd for IoPipe {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        let desc = FileDesc::from_raw_fd(fd);
        Self(desc)
    }
}

impl IntoRawFd for IoPipe {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl AsFd for IoPipe {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl IntoInner<OwnedFd> for IoPipe {
    fn into_inner(self) -> OwnedFd {
        self.0.into_inner()
    }
}

impl FromInner<OwnedFd> for IoPipe {
    fn from_inner(owned_fd: OwnedFd) -> Self {
        Self(FileDesc::from_inner(owned_fd))
    }
}

pub fn read2(_p1: IoPipe, _v1: &mut Vec<u8>, _p2: IoPipe, _v2: &mut Vec<u8>) -> io::Result<()> {
    todo!()
}
