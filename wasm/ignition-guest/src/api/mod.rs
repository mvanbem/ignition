use core::convert::TryInto;
use core::ffi::c_void;
use core::time::Duration;

pub(crate) mod sys;
pub(crate) mod wait;

use crate::runtime::reactor;

use self::wait::wait;

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

pub async fn impulse() {
    let task_id = reactor::new_task();

    // SAFETY: No special considerations.
    unsafe { sys::impulse(task_id) };

    wait(task_id).await;
}

pub async fn sleep(duration: Duration) {
    let task_id = reactor::new_task();
    let usec = duration.as_micros().try_into().unwrap();

    // SAFETY: No special considerations.
    unsafe { sys::sleep(task_id, usec) };

    wait(task_id).await;
}
