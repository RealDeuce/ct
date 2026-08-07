//! Cross-platform calling-thread CPU-time measurement.

unsafe extern "C" {
    fn ct_thread_cpu_time_ns(nanoseconds: *mut u64) -> std::ffi::c_int;
}

/// Return CPU nanoseconds consumed by the calling OS thread.
pub fn thread_cpu_time_ns() -> Option<u64> {
    let mut value = 0_u64;
    // SAFETY: `value` is a valid writable u64 for the duration of the call.
    let result = unsafe { ct_thread_cpu_time_ns(&mut value) };
    (result == 0).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calling_thread_cpu_clock_is_monotonic_when_available() {
        let before = thread_cpu_time_ns();
        let mut accumulator = 0_u64;
        for value in 0..10_000 {
            accumulator = accumulator.wrapping_add(value);
        }
        std::hint::black_box(accumulator);
        let after = thread_cpu_time_ns();
        if let (Some(before), Some(after)) = (before, after) {
            assert!(after >= before);
        }
    }
}
