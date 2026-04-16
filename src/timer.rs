//! Timer API.
//! Works by registering the next timer deadline to be hit and using it as a timeout
//! when polling for HAL external events.
//! Each timer is a future, so it gets polled in the polling loop.
//! When polled, all timers will compare their deadline value to the currently registered
//! nearest deadline to only keep the very next one and use it as a timout in the host poll
//! (blocking) function.
use std::{
    cell::RefCell,
    collections::BinaryHeap,
    task::{Poll, Waker},
    time::Instant,
};

pub fn sleep(duration: boppo_core::ShortDuration) -> Timer {
    Timer {
        end: Instant::now() + duration.as_std(),
    }
}

thread_local! {
    static TIMERS: RefCell<BinaryHeap<TimerWithWaker>> = RefCell::new(BinaryHeap::new());
}

pub struct Timer {
    end: Instant,
}

pub struct TimerWithWaker {
    end: Instant,
    waker: Waker,
}

impl Future for Timer {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if Instant::now() >= self.end {
            //Timer is ready
            Poll::Ready(())
        } else {
            // On next poll, wait at most this amount of time
            register_next_deadline(self.end, cx.waker().clone());
            Poll::Pending
        }
    }
}

// reverse ordering for TimerWithWaker based on end so that the nearest instant is on top of the heap
// This requires custom Ord, so partial_ord, partial_eq and eq to be implemented as well
impl PartialEq for TimerWithWaker {
    fn eq(&self, other: &Self) -> bool {
        self.end == other.end
    }
}

impl Eq for TimerWithWaker {}

impl PartialOrd for TimerWithWaker {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.end.partial_cmp(&self.end)
    }
}

impl Ord for TimerWithWaker {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.end.cmp(&self.end)
    }
}

/// Removes and wakes expired timers
/// This is meant to be called between poll loop iterations so that
/// it stays relevant (polling updates it)
pub fn wake_and_clean_expired_timers() {
    TIMERS.with(|cell| {
        let mut heap = cell.borrow_mut();
        let now = Instant::now();
        while heap.peek().map_or(false, |e| e.end < now) {
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
            return 0;
        };
        (timer_with_waker
            .end
            .saturating_duration_since(Instant::now())
            .as_millis() as i32)
            .max(1)
    })
}

fn register_next_deadline(deadline: Instant, waker: Waker) {
    TIMERS.with(|cell| {
        let mut heap = cell.borrow_mut();
        heap.push(TimerWithWaker {
            end: deadline,
            waker,
        });
    })
}
