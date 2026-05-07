use std::{cell::RefCell, collections::BinaryHeap, sync::OnceLock, task::Waker, time::Instant};

use embassy_time_driver::Driver;

#[cfg(feature = "wasm_client")]
use embassy_time_driver::time_driver_impl;

thread_local! {
    static TIMERS: RefCell<BinaryHeap<TimerWithWaker>> = const { RefCell::new(BinaryHeap::new()) };
}

static START: OnceLock<Instant> = OnceLock::new();

pub struct TimerWithWaker {
    at: u64,
    waker: Waker,
}

// reverse ordering for TimerWithWaker based on end so that the nearest instant is on top of the heap
// This requires custom Ord, so partial_ord, partial_eq and eq to be implemented as well
impl PartialEq for TimerWithWaker {
    fn eq(&self, other: &Self) -> bool {
        self.at == other.at
    }
}

impl Eq for TimerWithWaker {}

impl PartialOrd for TimerWithWaker {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.at.partial_cmp(&self.at)
    }
}

impl Ord for TimerWithWaker {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.at.cmp(&self.at)
    }
}

/// Timer driver implementation for embassy-time
struct BoppoWasmDriver;

impl Driver for BoppoWasmDriver {
    fn now(&self) -> u64 {
        START.get_or_init(Instant::now).elapsed().as_micros() as u64
    }

    fn schedule_wake(&self, at: u64, waker: &Waker) {
        TIMERS.with(|cell| {
            cell.borrow_mut().push(TimerWithWaker {
                at,
                waker: waker.clone(),
            });
        })
    }
}

#[cfg(feature = "wasm_client")]
time_driver_impl!(static DRIVER : BoppoWasmDriver = BoppoWasmDriver);

/// Removes and wakes expired timers
/// This is meant to be called between poll loop iterations so that
/// it stays relevant (polling updates it)
pub fn wake_and_clean_expired_timers() {
    TIMERS.with(|cell| {
        let mut heap = cell.borrow_mut();
        let now = BoppoWasmDriver.now();
        while heap.peek().is_some_and(|e| e.at <= now) {
            if let Some(timer_with_waker) = heap.pop() {
                timer_with_waker.waker.wake();
            } else {
                break;
            }
        }
    });
}

/// Returns the timeout for the next boppo_wasm_poll function, which
/// is the remaining time in milliseconds before the next deadline.
pub fn next_timeout() -> i32 {
    TIMERS.with(|cell| {
        let heap = cell.borrow();
        let Some(timer_with_waker) = heap.peek() else {
            return -1;
        };
        let now = BoppoWasmDriver.now();
        if timer_with_waker.at <= now {
            return 0;
        }

        (((timer_with_waker.at - now) / 1000).min(i32::MAX as u64) as i32).max(1)
    })
}
