//! Module relating to the measurement of time.

use crate::{Errno, SyscallNum, syscall_result};

/// The different clocks which can be used with [`clock_time`].
#[repr(usize)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum GetTimeClock {
    /// A settable system-wide clock that measures real (i.e., wall-clock) time.
    Realtime,
    /// A non-settable version of [`Self::Realtime`].
    RealtimeAlarm,
    /// A faster but less precise version of [`Self::Realtime`]. Not settable.
    RealtimeCoarse,
    /// A non-settable system-wide clock derived from wall-clock time but ignoring leap seconds.
    /// (International Atomic Time).
    Tai,
    /// A non-settable system-wide clock that measures the number of seconds that the system has
    /// been running since it was booted.
    ///
    /// This clock guarantees that the time returned by non-consecutive calls will not go
    /// backwards.
    Monotonic,
    /// A faster but less precise version of [`Self::Monotonic`].
    MonotonicCoarse,
    /// Similar to [`Self::Monotonic`], but is a raw hardware-based time unaffected by time
    /// adjustments. Does not measure time when the system is suspended.
    MonotonicRaw,
    /// Identical to [`Self::Monotonic`], but also includes any time that the system is suspended.
    BootTime,
    /// Like [`Self::BootTime`].
    BootTimeAlarm,
    /// Measures CPU time consumed by all threads on this process. Non-settable.
    ProcessCpuTime,
    /// Measures CPU time consumed by this thread. Non-settable.
    ThreadCpuTime,
}

/// The number of seconds and nanoseconds since a particular point in time.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Timespec {
    /// The number of seconds which have passed since a particular point of time.
    pub secs: i64,
    /// The number of nanoseconds which have passed since the last second.
    pub nanos: i64,
}
impl PartialOrd for Timespec {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Timespec {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.secs
            .cmp(&other.secs)
            .then(self.nanos.cmp(&other.nanos))
    }
}

/// Gets the [`Timespec`] associated with the given [`GetTimeClock`].
///
/// Wrapper around the
/// [`clock_gettime`](https://www.man7.org/linux/man-pages/man3/clock_gettime.3.html) Linux system
/// call.
///
/// # Errors
///
/// This function propagates any [`Errno`]s incurred by the underlying call to `clock_gettime`.
pub fn clock_time(clock: GetTimeClock) -> Result<Timespec, Errno> {
    let mut ts = Timespec::default();

    // SAFETY: The number and type of the system call arguments are correct. The `Timespec` struct
    // matches the expected size and alignment. The `Timespec` raw pointer is valid for the whole
    // duration of the syscall. The `GetTimeClock` enum restricts the `which_clock` argument to
    // valid values.
    unsafe {
        syscall_result!(
            SyscallNum::ClockGettime,
            clock as usize,
            (&raw mut ts) as usize
        )?;
    }

    Ok(ts)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use core::time::Duration;

    use super::*;
    use crate::thread::sleep;

    #[test_case]
    fn cmp_hw_time() {
        let time_1 = clock_time(GetTimeClock::Monotonic).unwrap();
        sleep(&Duration::from_secs(2)).unwrap();
        let time_2 = clock_time(GetTimeClock::Monotonic).unwrap();
        // Give a little breathing room as far as accuracy is concerned, we just want to make sure
        // it's getting *some* time
        assert!((time_2.secs - time_1.secs) > 0);
        assert!((time_2.secs - time_1.secs) < 4);
        assert!(time_2 > time_1);
    }

    #[test_case]
    fn timespec_cmp() {
        let ts = Timespec {
            secs: 100,
            nanos: 100,
        };
        let ts_more_secs = Timespec {
            secs: 1000,
            nanos: 10,
        };
        let ts_more_nanos = Timespec {
            secs: 10,
            nanos: 1000,
        };
        let ts_eq = Timespec {
            secs: 100,
            nanos: 100,
        };

        assert!(ts < ts_more_secs);
        assert!(ts > ts_more_nanos);
        assert!(ts == ts_eq);
    }
}
