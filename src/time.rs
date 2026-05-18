//! `std::time` shim.
//!
//! `Duration` re-exports `core::time::Duration` unchanged. `Instant` and
//! `SystemTime` defer to the host kernel via two `extern "C"` symbols that
//! the kernel must define:
//!
//! ```ignore
//! #[no_mangle]
//! extern "C" fn macula_std_monotonic_ns() -> u64;
//!
//! #[no_mangle]
//! extern "C" fn macula_std_wallclock_ns() -> u64;
//! ```
//!
//! `monotonic_ns` returns nanoseconds since an unspecified epoch that does
//! not run backwards; backs `Instant`. `wallclock_ns` returns nanoseconds
//! since the Unix epoch; backs `SystemTime`. If the kernel cannot supply a
//! wall-clock (no RTC, no NTP), `wallclock_ns` may return `0`; callers will
//! see all `SystemTime` values at the Unix epoch. TLS certificate validation
//! will then fail any time-dependent check, which is the correct failure
//! mode rather than a silent pass.

pub use core::time::Duration;

extern "C" {
    fn macula_std_monotonic_ns() -> u64;
    fn macula_std_wallclock_ns() -> u64;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant {
    ns: u64,
}

impl Instant {
    pub fn now() -> Self {
        Self {
            ns: unsafe { macula_std_monotonic_ns() },
        }
    }

    pub fn duration_since(&self, earlier: Self) -> Duration {
        Duration::from_nanos(self.ns.saturating_sub(earlier.ns))
    }

    pub fn checked_duration_since(&self, earlier: Self) -> Option<Duration> {
        self.ns
            .checked_sub(earlier.ns)
            .map(Duration::from_nanos)
    }

    pub fn saturating_duration_since(&self, earlier: Self) -> Duration {
        self.duration_since(earlier)
    }

    pub fn elapsed(&self) -> Duration {
        Self::now().duration_since(*self)
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        self.ns
            .checked_add(duration.as_nanos() as u64)
            .map(|ns| Self { ns })
    }

    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        self.ns
            .checked_sub(duration.as_nanos() as u64)
            .map(|ns| Self { ns })
    }
}

impl core::ops::Add<Duration> for Instant {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self {
        self.checked_add(rhs).expect("overflow when adding duration to instant")
    }
}

impl core::ops::AddAssign<Duration> for Instant {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl core::ops::Sub<Duration> for Instant {
    type Output = Self;
    fn sub(self, rhs: Duration) -> Self {
        self.checked_sub(rhs).expect("overflow when subtracting duration from instant")
    }
}

impl core::ops::SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

impl core::ops::Sub<Instant> for Instant {
    type Output = Duration;
    fn sub(self, rhs: Instant) -> Duration {
        self.duration_since(rhs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTime {
    ns: u64,
}

pub const UNIX_EPOCH: SystemTime = SystemTime { ns: 0 };

#[derive(Clone, Debug)]
pub struct SystemTimeError(Duration);

impl SystemTimeError {
    pub fn duration(&self) -> Duration {
        self.0
    }
}

impl core::fmt::Display for SystemTimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "second time provided was later than self")
    }
}

impl core::error::Error for SystemTimeError {}

impl SystemTime {
    pub const UNIX_EPOCH: SystemTime = UNIX_EPOCH;

    pub fn now() -> Self {
        Self {
            ns: unsafe { macula_std_wallclock_ns() },
        }
    }

    pub fn duration_since(&self, earlier: Self) -> Result<Duration, SystemTimeError> {
        self.ns
            .checked_sub(earlier.ns)
            .map(Duration::from_nanos)
            .ok_or_else(|| {
                SystemTimeError(Duration::from_nanos(earlier.ns.saturating_sub(self.ns)))
            })
    }

    pub fn elapsed(&self) -> Result<Duration, SystemTimeError> {
        Self::now().duration_since(*self)
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        self.ns
            .checked_add(duration.as_nanos() as u64)
            .map(|ns| Self { ns })
    }

    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        self.ns
            .checked_sub(duration.as_nanos() as u64)
            .map(|ns| Self { ns })
    }
}

impl core::ops::Add<Duration> for SystemTime {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self {
        self.checked_add(rhs).expect("overflow when adding duration to system time")
    }
}

impl core::ops::Sub<Duration> for SystemTime {
    type Output = Self;
    fn sub(self, rhs: Duration) -> Self {
        self.checked_sub(rhs).expect("overflow when subtracting duration from system time")
    }
}
