pub use super::common::Args;
use crate::ffi::OsString;

pub fn args() -> Args {
    let moturus_args = moto_rt::process::args(); // Vec<String>
    let mut rust_args = alloc::vec::Vec::new();

    for arg in moturus_args {
        rust_args.push(OsString::from(arg));
    }

    Args::new(rust_args)
}
