use std::mem::MaybeUninit;

use crate::api::sys;
use crate::api::wait::wait;
use crate::runtime::reactor;

pub struct ReadHandle {
    io: sys::IoHandle,
}

impl ReadHandle {
    pub(crate) fn from_raw(io: sys::IoHandle) -> Self {
        Self { io }
    }

    pub async fn read(&self, buf: &mut [u8]) -> usize {
        let task_id = reactor::new_task();
        let mut n: MaybeUninit<usize> = MaybeUninit::uninit();

        let result = unsafe {
            sys::io_read(
                task_id,
                self.io,
                buf.as_mut_ptr(),
                buf.len(),
                n.as_mut_ptr(),
            )
        };
        if result == 0 {
            // Completed synchronously.
            unsafe { n.assume_init() }
        } else {
            // Will complete asynchronously.
            wait(task_id).await
        }
    }

    pub async fn read_exact(&self, mut buf: &mut [u8]) {
        while !buf.is_empty() {
            let n = self.read(buf).await;
            assert!(n > 0 && n < buf.len());
            buf = &mut buf[n..];
        }
    }

    pub async fn read_to_end(&self) -> Vec<u8> {
        // TODO: Tune this!
        const RESERVATION_SIZE: usize = 32;
        let mut buf = Vec::with_capacity(RESERVATION_SIZE);
        let mut len = 0;
        loop {
            if len == buf.len() {
                buf.resize(len + RESERVATION_SIZE, 0);
            }
            let n = self.read(&mut buf[len..]).await;
            len += n;
            if n == 0 {
                buf.resize(len, 0);
                return buf;
            }
        }
    }
}

impl Drop for ReadHandle {
    fn drop(&mut self) {
        // SAFETY: No special considerations.
        unsafe { sys::io_close(self.io) }
    }
}

pub struct WriteHandle {
    io: sys::IoHandle,
}

impl WriteHandle {
    pub(crate) fn from_raw(io: sys::IoHandle) -> Self {
        Self { io }
    }

    pub async fn write(&self, buf: &[u8]) -> usize {
        let task_id = reactor::new_task();
        let mut n: MaybeUninit<usize> = MaybeUninit::uninit();

        let result =
            unsafe { sys::io_write(task_id, self.io, buf.as_ptr(), buf.len(), n.as_mut_ptr()) };
        if result == 0 {
            // Completed synchronously.
            unsafe { n.assume_init() }
        } else {
            // Will complete asynchronously.
            wait(task_id).await
        }
    }

    pub async fn write_all(&self, mut buf: &[u8]) {
        while !buf.is_empty() {
            let n = self.write(buf).await;
            assert!(n > 0 && n <= buf.len());
            buf = &buf[n..];
        }
    }
}

impl Drop for WriteHandle {
    fn drop(&mut self) {
        // SAFETY: No special considerations.
        unsafe { sys::io_close(self.io) }
    }
}
