#![unstable(reason = "not public", issue = "none", feature = "fd")]
// use crate::cmp;
// use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut, Read};
use crate::io::{self, BorrowedCursor, IoSliceMut, Read};
use crate::os::fd::RawFd;
use crate::os::fd::{AsFd, AsRawFd, BorrowedFd};
use crate::os::fd::FromRawFd;
use crate::os::fd::IntoRawFd;
use crate::os::fd::OwnedFd;
use crate::sys_common::{AsInner, FromInner, IntoInner};
use super::map_moturus_error;

#[derive(Debug)]
pub struct FileDesc(OwnedFd);

impl FileDesc {
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        moto_rt::fs::read(self.as_raw_fd(), buf).map_err(map_moturus_error)
    }

    pub fn read_buf(&self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        crate::io::default_read_buf(|buf| self.read(buf), cursor)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        io::default_read_vectored(|b| self.read(b), bufs)
    }

    #[inline]
    pub fn is_read_vectored(&self) -> bool {
        false
    }

    /*
    #[inline]
    pub fn try_clone(&self) -> io::Result<Self> {
        self.duplicate()
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut me = self;
        (&mut me).read_to_end(buf)
    }


    #[cfg(any(
        target_os = "aix",
        target_os = "dragonfly", // DragonFly 1.5
        target_os = "emscripten",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "hurd",
        target_os = "illumos",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd", // OpenBSD 2.7
    ))]
    pub fn read_vectored_at(&self, bufs: &mut [IoSliceMut<'_>], offset: u64) -> io::Result<usize> {
        let ret = cvt(unsafe {
            libc::preadv(
                self.as_raw_fd(),
                bufs.as_mut_ptr() as *mut libc::iovec as *const libc::iovec,
                cmp::min(bufs.len(), max_iov()) as libc::c_int,
                offset as _,
            )
        })?;
        Ok(ret as usize)
    }

    #[cfg(not(any(
        target_os = "aix",
        target_os = "android",
        target_os = "dragonfly",
        target_os = "emscripten",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "hurd",
        target_os = "illumos",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd",
        target_vendor = "apple",
    )))]
    pub fn read_vectored_at(&self, bufs: &mut [IoSliceMut<'_>], offset: u64) -> io::Result<usize> {
        io::default_read_vectored(|b| self.read_at(b, offset), bufs)
    }

    // We support some old Android versions that do not have `preadv` in libc,
    // so we use weak linkage and fallback to a direct syscall if not available.
    //
    // On 32-bit targets, we don't want to deal with weird ABI issues around
    // passing 64-bits parameters to syscalls, so we fallback to the default
    // implementation if `preadv` is not available.
    #[cfg(all(target_os = "android", target_pointer_width = "64"))]
    pub fn read_vectored_at(&self, bufs: &mut [IoSliceMut<'_>], offset: u64) -> io::Result<usize> {
        super::weak::syscall! {
            fn preadv(
                fd: libc::c_int,
                iovec: *const libc::iovec,
                n_iovec: libc::c_int,
                offset: off64_t
            ) -> isize
        }

        let ret = cvt(unsafe {
            preadv(
                self.as_raw_fd(),
                bufs.as_mut_ptr() as *mut libc::iovec as *const libc::iovec,
                cmp::min(bufs.len(), max_iov()) as libc::c_int,
                offset as _,
            )
        })?;
        Ok(ret as usize)
    }

    #[cfg(all(target_os = "android", target_pointer_width = "32"))]
    pub fn read_vectored_at(&self, bufs: &mut [IoSliceMut<'_>], offset: u64) -> io::Result<usize> {
        super::weak::weak!(fn preadv64(libc::c_int, *const libc::iovec, libc::c_int, off64_t) -> isize);

        match preadv64.get() {
            Some(preadv) => {
                let ret = cvt(unsafe {
                    preadv(
                        self.as_raw_fd(),
                        bufs.as_mut_ptr() as *mut libc::iovec as *const libc::iovec,
                        cmp::min(bufs.len(), max_iov()) as libc::c_int,
                        offset as _,
                    )
                })?;
                Ok(ret as usize)
            }
            None => io::default_read_vectored(|b| self.read_at(b, offset), bufs),
        }
    }

    // We support old MacOS, iOS, watchOS, tvOS and visionOS. `preadv` was added in the following
    // Apple OS versions:
    // ios 14.0
    // tvos 14.0
    // macos 11.0
    // watchos 7.0
    //
    // These versions may be newer than the minimum supported versions of OS's we support so we must
    // use "weak" linking.
    #[cfg(target_vendor = "apple")]
    pub fn read_vectored_at(&self, bufs: &mut [IoSliceMut<'_>], offset: u64) -> io::Result<usize> {
        super::weak::weak!(fn preadv(libc::c_int, *const libc::iovec, libc::c_int, off64_t) -> isize);

        match preadv.get() {
            Some(preadv) => {
                let ret = cvt(unsafe {
                    preadv(
                        self.as_raw_fd(),
                        bufs.as_mut_ptr() as *mut libc::iovec as *const libc::iovec,
                        cmp::min(bufs.len(), max_iov()) as libc::c_int,
                        offset as _,
                    )
                })?;
                Ok(ret as usize)
            }
            None => io::default_read_vectored(|b| self.read_at(b, offset), bufs),
        }
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let ret = cvt(unsafe {
            libc::write(
                self.as_raw_fd(),
                buf.as_ptr() as *const libc::c_void,
                cmp::min(buf.len(), READ_LIMIT),
            )
        })?;
        Ok(ret as usize)
    }

    #[cfg(not(any(
        target_os = "espidf",
        target_os = "horizon",
        target_os = "vita",
        target_os = "nuttx"
    )))]
    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let ret = cvt(unsafe {
            libc::writev(
                self.as_raw_fd(),
                bufs.as_ptr() as *const libc::iovec,
                cmp::min(bufs.len(), max_iov()) as libc::c_int,
            )
        })?;
        Ok(ret as usize)
    }

    #[cfg(any(
        target_os = "espidf",
        target_os = "horizon",
        target_os = "vita",
        target_os = "nuttx"
    ))]
    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        io::default_write_vectored(|b| self.write(b), bufs)
    }

    #[inline]
    pub fn is_write_vectored(&self) -> bool {
        cfg!(not(any(
            target_os = "espidf",
            target_os = "horizon",
            target_os = "vita",
            target_os = "nuttx"
        )))
    }

    #[cfg_attr(target_os = "vxworks", allow(unused_unsafe))]
    pub fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        #[cfg(not(any(
            all(target_os = "linux", not(target_env = "musl")),
            target_os = "android",
            target_os = "hurd"
        )))]
        use libc::pwrite as pwrite64;
        #[cfg(any(
            all(target_os = "linux", not(target_env = "musl")),
            target_os = "android",
            target_os = "hurd"
        ))]
        use libc::pwrite64;

        unsafe {
            cvt(pwrite64(
                self.as_raw_fd(),
                buf.as_ptr() as *const libc::c_void,
                cmp::min(buf.len(), READ_LIMIT),
                offset as off64_t,
            ))
            .map(|n| n as usize)
        }
    }

    #[cfg(any(
        target_os = "aix",
        target_os = "dragonfly", // DragonFly 1.5
        target_os = "emscripten",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "hurd",
        target_os = "illumos",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd", // OpenBSD 2.7
    ))]
    pub fn write_vectored_at(&self, bufs: &[IoSlice<'_>], offset: u64) -> io::Result<usize> {
        let ret = cvt(unsafe {
            libc::pwritev(
                self.as_raw_fd(),
                bufs.as_ptr() as *const libc::iovec,
                cmp::min(bufs.len(), max_iov()) as libc::c_int,
                offset as _,
            )
        })?;
        Ok(ret as usize)
    }

    #[cfg(not(any(
        target_os = "aix",
        target_os = "android",
        target_os = "dragonfly",
        target_os = "emscripten",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "hurd",
        target_os = "illumos",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd",
        target_vendor = "apple",
    )))]
    pub fn write_vectored_at(&self, bufs: &[IoSlice<'_>], offset: u64) -> io::Result<usize> {
        io::default_write_vectored(|b| self.write_at(b, offset), bufs)
    }

    // We support some old Android versions that do not have `pwritev` in libc,
    // so we use weak linkage and fallback to a direct syscall if not available.
    //
    // On 32-bit targets, we don't want to deal with weird ABI issues around
    // passing 64-bits parameters to syscalls, so we fallback to the default
    // implementation if `pwritev` is not available.
    #[cfg(all(target_os = "android", target_pointer_width = "64"))]
    pub fn write_vectored_at(&self, bufs: &[IoSlice<'_>], offset: u64) -> io::Result<usize> {
        super::weak::syscall! {
            fn pwritev(
                fd: libc::c_int,
                iovec: *const libc::iovec,
                n_iovec: libc::c_int,
                offset: off64_t
            ) -> isize
        }

        let ret = cvt(unsafe {
            pwritev(
                self.as_raw_fd(),
                bufs.as_ptr() as *const libc::iovec,
                cmp::min(bufs.len(), max_iov()) as libc::c_int,
                offset as _,
            )
        })?;
        Ok(ret as usize)
    }

    #[cfg(all(target_os = "android", target_pointer_width = "32"))]
    pub fn write_vectored_at(&self, bufs: &[IoSlice<'_>], offset: u64) -> io::Result<usize> {
        super::weak::weak!(fn pwritev64(libc::c_int, *const libc::iovec, libc::c_int, off64_t) -> isize);

        match pwritev64.get() {
            Some(pwritev) => {
                let ret = cvt(unsafe {
                    pwritev(
                        self.as_raw_fd(),
                        bufs.as_ptr() as *const libc::iovec,
                        cmp::min(bufs.len(), max_iov()) as libc::c_int,
                        offset as _,
                    )
                })?;
                Ok(ret as usize)
            }
            None => io::default_write_vectored(|b| self.write_at(b, offset), bufs),
        }
    }

    // We support old MacOS, iOS, watchOS, tvOS and visionOS. `pwritev` was added in the following
    // Apple OS versions:
    // ios 14.0
    // tvos 14.0
    // macos 11.0
    // watchos 7.0
    //
    // These versions may be newer than the minimum supported versions of OS's we support so we must
    // use "weak" linking.
    #[cfg(target_vendor = "apple")]
    pub fn write_vectored_at(&self, bufs: &[IoSlice<'_>], offset: u64) -> io::Result<usize> {
        super::weak::weak!(fn pwritev(libc::c_int, *const libc::iovec, libc::c_int, off64_t) -> isize);

        match pwritev.get() {
            Some(pwritev) => {
                let ret = cvt(unsafe {
                    pwritev(
                        self.as_raw_fd(),
                        bufs.as_ptr() as *const libc::iovec,
                        cmp::min(bufs.len(), max_iov()) as libc::c_int,
                        offset as _,
                    )
                })?;
                Ok(ret as usize)
            }
            None => io::default_write_vectored(|b| self.write_at(b, offset), bufs),
        }
    }
    */

    pub fn set_nonblocking(&self, _nonblocking: bool) -> io::Result<()> {
        todo!()
    }

    #[inline]
    pub fn duplicate(&self) -> io::Result<FileDesc> {
        // Ok(Self(self.0.try_clone()?))
        todo!()
    }
}

impl<'a> Read for &'a FileDesc {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf(cursor)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (**self).read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        (**self).is_read_vectored()
    }
}

impl AsInner<OwnedFd> for FileDesc {
    #[inline]
    fn as_inner(&self) -> &OwnedFd {
        &self.0
    }
}

impl IntoInner<OwnedFd> for FileDesc {
    fn into_inner(self) -> OwnedFd {
        self.0
    }
}

impl FromInner<OwnedFd> for FileDesc {
    fn from_inner(owned_fd: OwnedFd) -> Self {
        Self(owned_fd)
    }
}

impl AsFd for FileDesc {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for FileDesc {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl IntoRawFd for FileDesc {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl FromRawFd for FileDesc {
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self {
        Self(FromRawFd::from_raw_fd(raw_fd))
    }
}
