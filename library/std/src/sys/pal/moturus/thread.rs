use super::map_moturus_error;
use crate::ffi::CStr;
use crate::io;
use crate::num::NonZeroUsize;
use crate::time::{Duration, Instant};
pub const DEFAULT_MIN_STACK_SIZE: usize = 1024 * 256;

pub struct Thread {
    sys_thread: moto_rt::thread::ThreadHandle,
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

impl Thread {
    pub unsafe fn new(stack: usize, _name: Option<&str>, p: Box<dyn FnOnce()>) -> io::Result<Thread> {
        extern "C" fn __moto_rt_thread_fn(thread_arg: u64) {
            unsafe {
                Box::from_raw(
                    core::ptr::with_exposed_provenance::<Box<dyn FnOnce()>>(thread_arg as usize)
                        .cast_mut(),
                )();
            }
        }

        let thread_arg = Box::into_raw(Box::new(p)) as *mut _ as usize as u64;
        let sys_thread = moto_rt::thread::spawn(__moto_rt_thread_fn, stack, thread_arg)
            .map_err(map_moturus_error)?;
        Ok(Self { sys_thread })
    }

    pub fn yield_now() {
        moto_rt::thread::yield_now()
    }

    pub fn set_name(name: &CStr) {
        let bytes = name.to_bytes();
        if let Ok(s) = core::str::from_utf8(bytes) {
            let _ = moto_rt::thread::set_name(s);
        }
    }

    pub fn sleep(dur: Duration) {
        moto_rt::thread::sleep_until(moto_rt::time::Instant::now() + dur)
    }

    pub fn sleep_until(deadline: Instant) {
        let now = Instant::now();

        if let Some(delay) = deadline.checked_duration_since(now) {
            Self::sleep(delay);
        }
    }

    pub fn join(self) {
        assert!(moto_rt::thread::join(self.sys_thread) == moto_rt::E_OK)
    }
}

pub(crate) fn current_os_id() -> Option<u64> {
    None
}

pub fn available_parallelism() -> io::Result<NonZeroUsize> {
    Ok(unsafe { NonZeroUsize::new_unchecked(moto_rt::num_cpus()) })
}
