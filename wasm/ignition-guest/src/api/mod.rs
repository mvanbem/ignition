use core::ffi::c_void;

mod impulse;
mod sleep;
pub(crate) mod sys;

pub use self::impulse::impulse;
pub use self::sleep::sleep;

pub fn shutdown() {
    // SAFETY: No special considerations.
    unsafe { sys::shutdown() }
}

pub fn abort() -> ! {
    // SAFETY: No special considerations.
    unsafe { sys::abort() }
}

pub fn log(message: &str) {
    // SAFETY: `message` and `len` refer to a UTF-8 string.
    unsafe { sys::log(message.as_bytes().as_ptr() as *const c_void, message.len()) }
}
