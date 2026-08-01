use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

#[derive(Debug)]
pub struct Lock {
    _file: File,
}

impl Lock {
    pub fn new(p: &Path, wait: bool, create: bool, exclusive: bool) -> io::Result<Lock> {
        let mut options = OpenOptions::new();
        options.read(true);
        if create {
            options.create(true).write(true);
        }
        let file = options.open(p)?;

        match (wait, exclusive) {
            (true, true) => file.lock(),
            (true, false) => file.lock_shared(),
            (false, true) => file.try_lock().map_err(io::Error::from),
            (false, false) => file.try_lock_shared().map_err(io::Error::from),
        }?;

        Ok(Lock { _file: file })
    }

    pub fn error_unsupported(err: &io::Error) -> bool {
        err.kind() == io::ErrorKind::Unsupported
    }
}
