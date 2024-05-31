pub type Key = usize;

#[inline]
pub unsafe fn create(dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
    moto_runtime::tls::create(dtor)
}

#[inline]
pub unsafe fn set(key: Key, value: *mut u8) {
    moto_runtime::tls::set(key, value)
}

#[inline]
pub unsafe fn get(key: Key) -> *mut u8 {
    moto_runtime::tls::get(key)
}

#[inline]
pub unsafe fn destroy(key: Key) {
    moto_runtime::tls::destroy(key)
}
