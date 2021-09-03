use core::ffi::c_void;

mod sys {
    use core::ffi::c_void;

    #[link(wasm_import_module = "ignition")]
    extern "C" {
        pub fn abort() -> !;
        pub fn log(ptr: *const c_void, len: usize);
    }
}

pub fn abort() -> ! {
    // SAFETY: No special considerations.
    unsafe { sys::abort() }
}

pub fn log(message: &str) {
    // SAFETY: `message` and `len` refer to a UTF-8 string.
    unsafe { sys::log(message.as_bytes().as_ptr() as *const c_void, message.len()) }
}
