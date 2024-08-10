use super::map_moturus_error;
use crate::ffi::CStr;
use crate::io;
use crate::num::NonZeroUsize;
use crate::time::Duration;
pub const DEFAULT_MIN_STACK_SIZE: usize = 1024 * 256;

pub struct Thread {
    sys_thread: moto_runtime::thread::Thread,
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

impl Thread {
    pub unsafe fn new(stack: usize, p: Box<dyn FnOnce()>) -> io::Result<Thread> {
        let sys_thread = moto_runtime::thread::Thread::new(stack, p).map_err(map_moturus_error)?;
        Ok(Self { sys_thread })
    }

    pub fn yield_now() {
        moto_runtime::SysCpu::sched_yield();
    }

    pub fn set_name(name: &CStr) {
        let bytes = name.to_bytes();
        if let Ok(s) = core::str::from_utf8(bytes) {
            let _ = moto_runtime::set_current_thread_name(s);
        }
    }

    pub fn sleep(dur: Duration) {
        moto_runtime::thread::sleep(dur);
    }

    pub fn join(self) {
        self.sys_thread.join();
    }
}

pub fn available_parallelism() -> io::Result<NonZeroUsize> {
    Ok(unsafe { NonZeroUsize::new_unchecked(moto_runtime::num_cpus() as usize) })
}
