pub use super::common::Env;
use crate::ffi::{OsStr, OsString};
use crate::io;

/*
#[derive(Debug)]
pub struct Env {
    env: vec::IntoIter<(OsString, OsString)>,
}

// FIXME(https://github.com/rust-lang/rust/issues/114583): Remove this when <OsStr as Debug>::fmt matches <str as Debug>::fmt.
pub struct EnvStrDebug<'a> {
    slice: &'a [(OsString, OsString)],
}

impl fmt::Debug for EnvStrDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { slice } = self;
        f.debug_list()
            .entries(slice.iter().map(|(a, b)| (a.to_str().unwrap(), b.to_str().unwrap())))
            .finish()
    }
}

impl Env {
    pub fn str_debug(&self) -> impl fmt::Debug + '_ {
        let Self { env } = self;
        EnvStrDebug { slice: env.as_slice() }
    }
}

impl Iterator for Env {
    type Item = (OsString, OsString);
    fn next(&mut self) -> Option<(OsString, OsString)> {
        self.env.next()
    }
}
    */

pub fn env() -> Env {
    let moturus_env = moto_rt::process::env(); // Vec<(String, String)>
    let mut rust_env = vec![];

    for (k, v) in moturus_env {
        rust_env.push((OsString::from(k), OsString::from(v)));
    }

    Env::new(rust_env)
}

pub fn getenv(key: &OsStr) -> Option<OsString> {
    moto_rt::process::getenv(key.to_str().unwrap()).map(|s| OsString::from(s))
}

pub unsafe fn setenv(key: &OsStr, val: &OsStr) -> io::Result<()> {
    Ok(moto_rt::process::setenv(key.to_str().unwrap(), val.to_str().unwrap()))
}

pub unsafe fn unsetenv(key: &OsStr) -> io::Result<()> {
    Ok(moto_rt::process::unsetenv(key.to_str().unwrap()))
}
