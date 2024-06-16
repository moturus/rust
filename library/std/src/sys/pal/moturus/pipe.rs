use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut};
use crate::sys::unsupported;

use super::map_moturus_error;

pub struct AnonPipe {
    // For some reason there is feature(write_mt), so we need to
    // protect the pipe here.
    pub pipe_rt: crate::sync::Mutex<moto_runtime::sync_pipe::Pipe>,
}

impl AnonPipe {

    pub fn new(pipe_rt: moto_runtime::sync_pipe::Pipe) -> Self {
        Self { pipe_rt: crate::sync::Mutex::new(pipe_rt) }
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.pipe_rt.lock().unwrap().read(buf).map_err(map_moturus_error)
    }

    pub fn read_buf(&self, _buf: BorrowedCursor<'_>) -> io::Result<()> {
        unsupported()
    }

    pub fn read_vectored(&self, _bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        unsupported()
    }

    pub fn is_read_vectored(&self) -> bool {
        false
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.pipe_rt.lock().unwrap().write(buf).map_err(map_moturus_error)
    }

    pub fn write_vectored(&self, _bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        unsupported()
    }

    pub fn is_write_vectored(&self) -> bool {
        false
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.pipe_rt.lock().unwrap().read_to_end(buf).map_err(map_moturus_error)
    }
}

pub fn read2(_p1: AnonPipe, _v1: &mut Vec<u8>, _p2: AnonPipe, _v2: &mut Vec<u8>) -> io::Result<()> {
    todo!()
}
