use crate::ffi::CStr;
use crate::io;
use crate::num::NonZeroUsize;
use crate::ptr;
use crate::time::Duration;

pub struct Thread {
    handle: moto_runtime::SysHandle
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

pub const DEFAULT_MIN_STACK_SIZE: usize = 1024 * 256;

impl Thread {
    pub unsafe fn new(stack: usize, p: Box<dyn FnOnce()>) -> io::Result<Thread> {
        let thread_arg = Box::into_raw(Box::new(p)) as *mut _;
        return match moto_runtime::thread::spawn(stack, thread_fn as usize, thread_arg as usize) {
            Ok(handle) => Ok(Self{handle}),
            Err(_) => {
                drop(Box::from_raw(thread_arg));
                Err(io::const_io_error!(io::ErrorKind::Uncategorized, "Unable to create thread!"))
            }
        };

        extern "C" fn thread_fn(thread_arg: usize) {
            unsafe {
                Box::from_raw(ptr::with_exposed_provenance::<Box<dyn FnOnce()>>(thread_arg).cast_mut())();

                // TODO: run all destructors
                // super::thread_local_dtor::run_dtors();
            }
            moto_runtime::tls::thread_exiting();
            moto_runtime::thread::exit_self();  // Well-formed threads must exit this way.
        }
    }

    pub fn yield_now() {
        // do nothing
    }

    pub fn set_name(_name: &CStr) {
        // nope
    }

    pub fn sleep(dur: Duration) {
        moto_runtime::thread::sleep(dur);
    }

    pub fn join(self) {
        moto_runtime::thread::join(self.handle);
    }
}

pub fn available_parallelism() -> io::Result<NonZeroUsize> {
    Ok(unsafe { NonZeroUsize::new_unchecked(moto_runtime::num_cpus() as usize) })
}
