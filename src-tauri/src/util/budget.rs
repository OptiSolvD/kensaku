//! A byte budget for decoded image buffers that are alive at the same time.
//!
//! Image recognition holds a full decoded page in memory while it runs. The
//! cost is `width * height * 4` bytes, which is roughly 400 MB for a 100 MP
//! scan. Worker threads used to decode whenever they reached an image, so the
//! peak was "how many images are in flight times how big they are", and the
//! number of workers came from the CPU count, not from memory.
//!
//! The budget makes the peak explicit. A caller reserves the decoded size before
//! it decodes, and waits while the reservation would exceed the budget. The
//! reservation is returned when the [`Permit`] is dropped.
//!
//! Requests larger than the whole budget are clamped to the budget. Such a
//! request waits until nothing else is reserved and then runs alone. That keeps
//! an oversized image from deadlocking the queue while still making it the only
//! thing in memory.

use std::sync::{Condvar, Mutex, MutexGuard, OnceLock};

/// Default budget for decoded images in flight: 768 MiB.
///
/// Large enough for several typical photos or screenshots at once, small enough
/// that a 16 GB machine keeps most of its memory for everything else. Override
/// with `KENSAKU_DECODE_BUDGET_MB` (for example on a workstation that wants more
/// parallelism, or on a small machine that wants less).
pub const DEFAULT_DECODE_BUDGET_BYTES: u64 = 768 * 1024 * 1024;

/// Counting semaphore measured in bytes.
pub struct DecodeBudget {
    total: u64,
    in_use: Mutex<u64>,
    released: Condvar,
}

/// A reservation. Dropping it returns the bytes to the budget.
#[must_use = "dropping the permit immediately releases the reservation"]
pub struct Permit {
    budget: &'static DecodeBudget,
    bytes: u64,
}

impl DecodeBudget {
    /// A budget of `total` bytes. Zero is treated as one byte, so every request
    /// is still serialized rather than refused.
    pub fn new(total: u64) -> Self {
        Self {
            total: total.max(1),
            in_use: Mutex::new(0),
            released: Condvar::new(),
        }
    }

    /// Total capacity in bytes.
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Bytes currently reserved by live permits.
    pub fn in_use(&self) -> u64 {
        *self.lock()
    }

    /// Waits until `bytes` can be reserved, then returns the reservation.
    ///
    /// `bytes` is clamped to the budget, so this always returns. A zero request
    /// reserves one byte, which keeps accounting honest for empty inputs.
    pub fn acquire(&'static self, bytes: u64) -> Permit {
        let wanted = bytes.clamp(1, self.total);
        let mut in_use = self.lock();
        while *in_use + wanted > self.total {
            in_use = self
                .released
                .wait(in_use)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        *in_use += wanted;
        Permit {
            budget: self,
            bytes: wanted,
        }
    }

    fn lock(&self) -> MutexGuard<'_, u64> {
        // A panic while holding the lock leaves the counter as it was; there is
        // nothing to repair, so keep serving rather than cascading the panic.
        self.in_use.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn release(&self, bytes: u64) {
        let mut in_use = self.lock();
        *in_use = in_use.saturating_sub(bytes);
        drop(in_use);
        self.released.notify_all();
    }
}

impl Permit {
    /// Bytes this permit reserved (after clamping).
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        self.budget.release(self.bytes);
    }
}

/// The process-wide budget for decoded images.
pub fn decode_budget() -> &'static DecodeBudget {
    static BUDGET: OnceLock<DecodeBudget> = OnceLock::new();
    BUDGET.get_or_init(|| {
        let bytes = std::env::var("KENSAKU_DECODE_BUDGET_MB")
            .ok()
            .and_then(|raw| raw.trim().parse::<u64>().ok())
            .filter(|mb| *mb > 0)
            .map(|mb| mb * 1024 * 1024)
            .unwrap_or(DEFAULT_DECODE_BUDGET_BYTES);
        DecodeBudget::new(bytes)
    })
}

/// Bytes an RGBA8 decode of `width` x `height` will occupy.
pub fn rgba_bytes(width: u32, height: u32) -> u64 {
    u64::from(width) * u64::from(height) * 4
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    /// Leaks a budget so the `'static` lifetime the permit needs is satisfied in
    /// tests. Test processes are short-lived, so the leak is harmless.
    fn leak(total: u64) -> &'static DecodeBudget {
        Box::leak(Box::new(DecodeBudget::new(total)))
    }

    #[test]
    fn reserves_and_releases_on_drop() {
        let budget = leak(100);
        let a = budget.acquire(40);
        assert_eq!(budget.in_use(), 40);
        let b = budget.acquire(60);
        assert_eq!(budget.in_use(), 100);
        drop(a);
        assert_eq!(budget.in_use(), 60);
        drop(b);
        assert_eq!(budget.in_use(), 0);
    }

    #[test]
    fn oversized_request_is_clamped_and_runs_alone() {
        let budget = leak(100);
        let huge = budget.acquire(10_000);
        assert_eq!(huge.bytes(), 100);
        assert_eq!(budget.in_use(), 100);
        drop(huge);
        assert_eq!(budget.in_use(), 0);
    }

    #[test]
    fn zero_request_still_reserves_one_byte() {
        let budget = leak(100);
        let p = budget.acquire(0);
        assert_eq!(p.bytes(), 1);
        drop(p);
    }

    #[test]
    fn concurrent_holders_never_exceed_the_budget() {
        // Each thread reserves 30 of a 100-byte budget. At most three can hold
        // a permit at once, so the observed peak must never pass 100.
        let budget = leak(100);
        let live = Arc::new(AtomicU64::new(0));
        let peak = Arc::new(AtomicU64::new(0));

        let handles: Vec<_> = (0..12)
            .map(|_| {
                let live = Arc::clone(&live);
                let peak = Arc::clone(&peak);
                thread::spawn(move || {
                    let permit = budget.acquire(30);
                    let now = live.fetch_add(permit.bytes(), Ordering::SeqCst) + permit.bytes();
                    peak.fetch_max(now, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(5));
                    live.fetch_sub(permit.bytes(), Ordering::SeqCst);
                    drop(permit);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
        assert!(
            peak.load(Ordering::SeqCst) <= 100,
            "peak reserved bytes {} exceeded the budget",
            peak.load(Ordering::SeqCst)
        );
        assert_eq!(budget.in_use(), 0);
    }

    #[test]
    fn a_waiter_proceeds_once_space_is_released() {
        let budget = leak(100);
        let held = budget.acquire(100);
        let waiter = thread::spawn(move || {
            let started = std::time::Instant::now();
            let _p = budget.acquire(50);
            started.elapsed()
        });
        thread::sleep(Duration::from_millis(30));
        drop(held);
        let waited = waiter.join().unwrap();
        assert!(waited >= Duration::from_millis(20), "waiter did not block");
    }

    #[test]
    fn rgba_bytes_is_four_per_pixel() {
        assert_eq!(rgba_bytes(10, 10), 400);
        assert_eq!(rgba_bytes(10_000, 10_000), 400_000_000);
    }
}
