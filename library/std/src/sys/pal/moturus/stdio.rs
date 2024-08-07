use crate::io;
use moto_runtime::stdio::*;

pub struct Stdin {
    rt: StdinRt,
}

pub struct Stdout {
    rt: StdoutRt,
}

pub struct Stderr {
    rt: StderrRt,
}

impl Stdin {
    pub const fn new() -> Self { Self { rt: StdinRt::new() } }
}

impl crate::sealed::Sealed for Stdin {}

impl crate::io::IsTerminal for Stdin {
    fn is_terminal(&self) -> bool {
        false
    }
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.rt.read(buf).map_err(super::map_moturus_error)
    }
}

impl Stdout {
    pub const fn new() -> Self { Self { rt: StdoutRt::new() } }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.rt.write(buf).map_err(super::map_moturus_error)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.rt.flush().map_err(super::map_moturus_error)
    }
}

impl Stderr {
    pub const fn new() -> Self { Self { rt: StderrRt::new() } }
}

impl io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.rt.write(buf).map_err(super::map_moturus_error)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.rt.flush().map_err(super::map_moturus_error)
    }
}

pub const STDIN_BUF_SIZE: usize = 64;

pub fn is_ebadf(_err: &io::Error) -> bool {
    true
}

pub fn panic_output() -> Option<impl io::Write> {
    Some(Stderr::new())
}
