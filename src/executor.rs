//! Since edge executor is scheduled to be removed from boppo_core,
//! We should provide it with an async executor in WASM.
//! For now, it mimics what happens in boppo_core.

use std::{
    pin::pin,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use boppo_core::log;
use edge_executor::{LocalExecutor, Task};

use crate::{
    HostEvent,
    host_ffi::{buttons::register_event, host_event::boppo_wasm_poll},
};

use crate::timer::{next_timeout, wake_and_clean_expired_timers};

const MAX_TASKS: usize = 32;
static EXECUTOR: AtomicPtr<LocalExecutor<'static, MAX_TASKS>> =
    AtomicPtr::new(std::ptr::null_mut());

/// Initializes the executor.
pub fn init() {
    let executor = Box::leak(Box::new(LocalExecutor::<MAX_TASKS>::new()));
    EXECUTOR.store(executor as *mut _, std::sync::atomic::Ordering::SeqCst);
    boppo_core::hal::set_executor(executor);
}

/// Spawns an asynchronous task
/// Should be dropped and stopped when the parent thread is dropped.
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

/// Set to true when the signal waker is called, meaning edge_executor has at least 1
/// task ready to run.
static WOKEN: AtomicBool = AtomicBool::new(false);

/// The signal waker is passed as the outer Context to executor().run().
/// This is what will let the executor signal that there is a task ready.
/// In this case, running the polling a second time will wake those tasks.
fn signal_waker() -> Waker {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        |_| RawWaker::new(std::ptr::null(), &VTABLE),
        |_| {
            WOKEN.store(true, Ordering::Relaxed);
        },
        |_| {
            WOKEN.store(true, Ordering::Relaxed);
        },
        |_| {},
    );
    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
}

pub fn block_on<T>(fut: impl Future<Output = T>) -> T {
    let mut top = pin!(executor().run(fut));
    let waker = signal_waker();
    let mut cx = Context::from_waker(&waker);

    loop {
        // Set WOKEN to false at each polling loop iteration so that it skips
        // boppo_wasm_poll only if an executor-handled task is ready (timers)
        WOKEN.store(false, Ordering::Relaxed);

        // Poll all ready tasks and the main future. Any task that becomes ready
        // during this poll (e.g. via spawn) will call the signal waker, which in turn sets
        // WOKEN to true.
        if let Poll::Ready(v) = top.as_mut().poll(&mut cx) {
            return v;
        }

        // If WOKEN was set during the poll, at least one task is queued and ready.
        // In this case we re-poll immediately, skipping boppo_wasm_poll:
        // this ensures timers are set.
        if WOKEN.load(Ordering::Relaxed) {
            continue;
        }

        // No tasks are ready. Block until the next host event or timer.
        // If no timer is set up, next_timeout returns -1, which polls indefinitely.
        let raw: Result<HostEvent, String> = unsafe { boppo_wasm_poll(next_timeout()) }.try_into();
        match raw {
            Err(e) => log::error!("Received unrecognized event from host : {e}"),
            Ok(HostEvent::Button(e)) => register_event(e),
            Ok(HostEvent::Timeout) => wake_and_clean_expired_timers(),
            _ => {
                // Anything else means the host disconnected, which should exit the activity.
                std::process::exit(0);
            }
        }
    }
}
