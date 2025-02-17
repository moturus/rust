use crate::io;

pub struct Stdin {}

pub struct Stdout {}

pub struct Stderr {}

impl Stdin {
    pub const fn new() -> Self {
        Self {}
    }
}

impl crate::sealed::Sealed for Stdin {}

impl crate::io::IsTerminal for Stdin {
    fn is_terminal(&self) -> bool {
        moto_rt::fs::is_terminal(moto_rt::FD_STDIN)
    }
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        moto_rt::fs::read(moto_rt::FD_STDIN, buf).map_err(super::map_moturus_error)
    }
}

impl Stdout {
    pub const fn new() -> Self {
        Self {}
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        moto_rt::fs::write(moto_rt::FD_STDOUT, buf).map_err(super::map_moturus_error)
    }

    fn flush(&mut self) -> io::Result<()> {
        moto_rt::fs::flush(moto_rt::FD_STDOUT).map_err(super::map_moturus_error)
    }
}

impl Stderr {
    pub const fn new() -> Self {
        Self {}
    }
}

impl io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        moto_rt::fs::write(moto_rt::FD_STDERR, buf).map_err(super::map_moturus_error)
    }

    fn flush(&mut self) -> io::Result<()> {
        moto_rt::fs::flush(moto_rt::FD_STDERR).map_err(super::map_moturus_error)
    }
}

pub const STDIN_BUF_SIZE: usize = 64;

pub fn is_ebadf(_err: &io::Error) -> bool {
    true
}

pub fn panic_output() -> Option<impl io::Write> {
    Some(Stderr::new())
}
