use super::map_moturus_error;
use crate::fmt;
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut};
use crate::sys::unsupported;

pub struct IoPipe {
    pub(crate) rt_fd: moto_rt::RtFd,
}

impl From<moto_rt::RtFd> for IoPipe {
    fn from(rt_fd: moto_rt::RtFd) -> IoPipe {
        IoPipe { rt_fd }
    }
}

impl fmt::Debug for IoPipe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IoPipe:{}", self.rt_fd)
    }
}

impl IoPipe {
    pub fn try_clone(&self) -> io::Result<Self> {
        todo!()
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        moto_rt::fs::read(self.rt_fd, buf).map_err(map_moturus_error)
    }

    pub fn read_buf(&self, _buf: BorrowedCursor<'_>) -> io::Result<()> {
        unsupported()
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        crate::io::default_read_vectored(|b| self.read(b), bufs)
    }

    pub fn is_read_vectored(&self) -> bool {
        false
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        moto_rt::fs::write(self.rt_fd, buf).map_err(map_moturus_error)
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

pub fn read2(_p1: IoPipe, _v1: &mut Vec<u8>, _p2: IoPipe, _v2: &mut Vec<u8>) -> io::Result<()> {
    todo!()
}
