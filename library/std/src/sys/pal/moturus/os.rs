use crate::error::Error as StdError;
use crate::ffi::{OsStr, OsString};
use crate::fmt;
use crate::io;
use crate::vec;
use crate::marker::PhantomData;
use crate::path::{self, PathBuf};

use super::map_moturus_error;

pub fn errno() -> i32 {
    // Why is this here? This belongs to libc.
    -1
}

pub fn error_string(_errno: i32) -> String {
    // Why is this here? This belongs to libc.
    "errno is a C/libc concept. Don't use it in Rust.".to_owned()
}

pub fn getcwd() -> io::Result<PathBuf> {
    // The CWD is a bad/outdated/unix design, from a single-threaded era:
    // concurrent changes to CWD lead to races. Applications/processes
    // should manage their CWD, not the OS.
    moto_runtime::fs::getcwd().map(|s| ->
        PathBuf { s.into() }).map_err(map_moturus_error)
}

pub fn chdir(path: &path::Path) -> io::Result<()> {
    // The CWD is a bad/outdated/unix design, from a single-threaded era:
    // concurrent changes to CWD lead to races. Applications/processes
    // should manage their CWD, not the OS.
    if let Some(path) = path.to_str() {
        moto_runtime::fs::chdir(path).map_err(map_moturus_error)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidFilename, ""))
    }
}

pub struct SplitPaths<'a>(!, PhantomData<&'a ()>);

pub fn split_paths(_unparsed: &OsStr) -> SplitPaths<'_> {
    panic!("unsupported")
}

impl<'a> Iterator for SplitPaths<'a> {
    type Item = PathBuf;
    fn next(&mut self) -> Option<PathBuf> {
        self.0
    }
}

#[derive(Debug)]
pub struct JoinPathsError;

pub fn join_paths<I, T>(_paths: I) -> Result<OsString, JoinPathsError>
where
    I: Iterator<Item = T>,
    T: AsRef<OsStr>,
{
    Err(JoinPathsError)
}

impl fmt::Display for JoinPathsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "not supported on this platform yet".fmt(f)
    }
}

impl StdError for JoinPathsError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "not supported on this platform yet"
    }
}

pub fn current_exe() -> io::Result<PathBuf> {
    if let Some(exe) = super::args::args().next() {
        Ok(exe.into())
    } else {
        Ok("<unknown>".into())
    }
}

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

pub fn env() -> Env {
    let moturus_env = moto_runtime::env::env();  // Vec<(&'static [u8], &'static [u8])>
    let mut rust_env = vec![];

    for (k, v) in moturus_env {
        rust_env.push((
            OsString::from(k),
            OsString::from(v)
        ));
    }

    Env { env: rust_env.into_iter() }
}

pub fn getenv(key: &OsStr) -> Option<OsString> {
    moto_runtime::env::getenv(key.to_str().unwrap()).map(|s| OsString::from(s))
}

pub fn setenv(key: &OsStr, val: &OsStr) -> io::Result<()> {
    Ok(moto_runtime::env::setenv(key.to_str().unwrap(), val.to_str().unwrap()))
}

pub fn unsetenv(key: &OsStr) -> io::Result<()> {
    Ok(moto_runtime::env::unsetenv(key.to_str().unwrap()))
}

pub fn temp_dir() -> PathBuf {
    PathBuf::from(moto_runtime::rt_api::TEMP_DIR)
}

pub fn home_dir() -> Option<PathBuf> {
    None
}

pub fn exit(code: i32) -> ! {
    moto_runtime::exit(code)
}

pub fn getpid() -> u32 {
    // Our pids are u64. Why does Rust mandate u32???
    panic!("no pids on this platform")
}
