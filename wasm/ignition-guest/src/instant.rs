use core::ops::Sub;
use core::time::Duration;

/// A measurement of a monotonically nondecreasing clock, analogous to [`std::time::Instant`].
pub struct Instant(u64);

impl Instant {
    pub fn now() -> Self {
        // SAFETY: No special considerations.
        Self(unsafe { crate::api::sys::monotonic_time() })
    }
}

impl Sub for Instant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Duration {
        Duration::from_micros(self.0.checked_sub(rhs.0).unwrap())
    }
}
