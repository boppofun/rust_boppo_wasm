//! Timer API.
//! Works by registering the next timer deadline to be hit and using it as a timeout
//! when polling for HAL external events.
//! Each timer is a future, so it gets polled in the polling loop.
//! When polled, all timers will compare their deadline value to the currently registered
//! nearest deadline to only keep the very next one and use it as a timout in the host poll
//! (blocking) function.
use std::{cell::RefCell, task::Poll, time::Instant};

pub fn sleep(duration: boppo_core::ShortDuration) -> Timer {
    Timer {
        end: Instant::now() + duration.as_std(),
    }
}

thread_local! {
    /// Next timer end to be hit by the executor.
    static NEXT_DEADLINE: RefCell<Option<Instant>> = RefCell::new(None);
}

pub struct Timer {
    end: Instant,
}

impl Future for Timer {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if Instant::now() >= self.end {
            //Timer is ready
            Poll::Ready(())
        } else {
            // On next poll, wait at most this amount of time
            register_next_deadline(self.end);
            Poll::Pending
        }
    }
}

/// Resets the next deadline.
/// This is meant to be called between poll loop iterations so that
/// it stays relevant (polling updates it)
pub(crate) fn reset_next_deadline() {
    NEXT_DEADLINE.with(|cell| *cell.borrow_mut() = None);
}

/// Returns the timeout for the next boppo_wasm_poll function, which
/// is the remaining time in milliseconds before the next deadline.
pub(crate) fn next_timeout() -> i32 {
    NEXT_DEADLINE.with(|cell| {
        if let Some(deadline) = *cell.borrow() {
            let remaining = deadline.saturating_duration_since(Instant::now());
            // Use 1 as a minimum since 0 means "indefinite" :
            // if it is 0, we're already late anyway
            (remaining.as_millis() as i32).max(1)
        } else {
            // No timeout is represented by 0 in the poll loop
            0
        }
    })
}

fn register_next_deadline(deadline: Instant) {
    NEXT_DEADLINE.with(|cell| {
        let mut current = cell.borrow_mut();
        match *current {
            None => *current = Some(deadline),

            Some(current_deadline) => {
                if deadline < current_deadline {
                    *current = Some(deadline);
                }
            }
        }
    })
}
