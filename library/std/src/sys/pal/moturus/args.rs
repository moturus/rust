use crate::ffi::OsString;
use crate::fmt;
use crate::vec;

pub struct Args {
    args: vec::IntoIter<OsString>,
}

pub fn args() -> Args {
    let moturus_args = moto_runtime::args::args();  // Vec<&'static [u8]>
    let mut rust_args = vec![];

    for arg in moturus_args {
        rust_args.push(OsString::from(arg));
    }

    Args { args: rust_args.into_iter() }
}

impl fmt::Debug for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().finish()
    }
}

impl Iterator for Args {
    type Item = OsString;

    fn next(&mut self) -> Option<Self::Item> {
        self.args.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.args.size_hint()
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        self.args.len()
    }
}

impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<OsString> {
        self.args.next_back()
    }
}
