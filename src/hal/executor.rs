//! Since edge executor is scheduled to be removed from boppo_core,
//! We should provide it with an async executor in WASM.
//! For now, it mimics what happens in boppo_core.

use std::{
    pin::pin,
    sync::atomic::AtomicPtr,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use edge_executor::{LocalExecutor, Task};

use crate::hal::{
    buttons::{boppo_wasm_poll, register_event},
    timer::{next_timeout, wake_and_clean_expired_timers},
};

const MAX_TASKS: usize = 32;
static EXECUTOR: AtomicPtr<LocalExecutor<'static, MAX_TASKS>> =
    AtomicPtr::new(std::ptr::null_mut());

pub fn init() {
    let executor = Box::leak(Box::new(LocalExecutor::<MAX_TASKS>::new()));
    EXECUTOR.store(executor as *mut _, std::sync::atomic::Ordering::SeqCst);
    boppo_core::hal::set_executor(executor);
}

/// Spawns an asynchronous task
///
/// TODO: automatically clean up activity tasks when an activity is ended
/// (e.g. home button press).
pub fn spawn<F, T>(fut: F) -> Task<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    executor().spawn(fut)
}

/// Gets the executor from the atomic pointer.
///
/// # Safety
/// This should not be called twice without dropping first,
/// so block_on should never be called from within block_on.
fn executor() -> &'static LocalExecutor<'static, MAX_TASKS> {
    unsafe { &*EXECUTOR.load(std::sync::atomic::Ordering::SeqCst) }
}

/// We need a waker for the Context API, in turn needed by the Future::poll API
/// Since the actual waiting for our polling function happens on the host
/// with its own executor, we just need a simple waker to satisfy the API.
///
/// # Safety
/// The pointer is null and no allocation happens even on cloning.
/// Most callbacks are no-ops, so no mutation happens on the waker.
fn noop_waker() -> Waker {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        |_| RawWaker::new(std::ptr::null(), &VTABLE),
        |_| {},
        |_| {},
        |_| {},
    );
    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
}

pub fn block_on<T>(fut: impl Future<Output = T>) -> T {
    let mut top = pin!(executor().run(fut));
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);

    loop {
        // Poll futures first so they can register their subscriptions (e.g. button
        // receivers) before we block waiting for the next event.
        // Waiting is done below on the native host thread during boppo_wasm_poll
        // which waits for the next button event
        if let Poll::Ready(v) = top.as_mut().poll(&mut cx) {
            return v;
        }
        let next_timeout = next_timeout();
        // Block until the next host event or next sleep deadline, then wake the futures above.
        let raw = unsafe { boppo_wasm_poll(next_timeout) };

        // The raw_wasm_code is an i32 representing a ButtonEvent if it's >= 0, a timeout if
        // it's equal to -1, and a closed channel if it's equal to -2 that should exit early.
        match raw {
            e if e >= 0 => {
                register_event(e);
            }
            -1 => {
                // Timeout.
                wake_and_clean_expired_timers();
            }
            _ => {
                // -2 or anything else : host channel got disconnected.
                std::process::exit(0);
            }
        }
    }
}
