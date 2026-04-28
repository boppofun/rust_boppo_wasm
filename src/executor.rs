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
    host_ffi::{audio::AUDIO_SENDER, buttons::broadcast_event, host_event::boppo_wasm_poll},
};

use crate::timer::{next_timeout, wake_and_clean_expired_timers};

const MAX_TASKS: usize = 32;
static EXECUTOR: AtomicPtr<LocalExecutor<'static, MAX_TASKS>> =
    AtomicPtr::new(std::ptr::null_mut());

/// Initializes the executor.
pub fn init() {
    let executor = Box::leak(Box::new(LocalExecutor::<MAX_TASKS>::new()));
    EXECUTOR.store(executor as *mut _, std::sync::atomic::Ordering::Relaxed);
    boppo_core::hal::set_executor(executor);
}

/// Spawns an asynchronous task
pub fn spawn<F, T>(fut: F) -> Task<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    executor().spawn(fut)
}

/// Gets the global WASM Executor.
///
/// # Safety
///
/// While LocalExecutor is !Sync, its "run" method uses Sync primitives
///
/// We are in a single-threaded context, so no data race can happen
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

#[doc(hidden)]
/// Blocks on a future.
/// Intended to be used to launch an async function from the main (sync) function of the module.
/// WARNING : this should not be called from inside the async activit without the risk for
/// logical errors.
/// It is only meant to be used for async runtime initialization.
pub fn internal_block_on<T>(fut: impl Future<Output = T>) -> T {
    let mut top = pin!(executor().run(fut));
    let waker = signal_waker();
    let mut cx = Context::from_waker(&waker);

    loop {
        // Reset before each poll so we can detect if any task wakes during this iteration.
        WOKEN.store(false, Ordering::Relaxed);

        // Poll all ready tasks and the main future. Any task that becomes ready
        // during this poll (e.g. via spawn) will call the signal waker, which in turn sets
        // WOKEN to true.
        if let Poll::Ready(v) = top.as_mut().poll(&mut cx) {
            return v;
        }

        // If tasks are ready, use timeout 0 (non-blocking) to avoid potential host event
        // starvation.
        // Otherwise block until the next event or timer (-1 if no timer is pending).
        let timeout = if WOKEN.load(Ordering::Relaxed) {
            0
        } else {
            next_timeout()
        };
        let raw: Result<HostEvent, String> = unsafe { boppo_wasm_poll(timeout) }.try_into();
        match raw {
            Err(e) => log::debug!("Received unrecognized event from host : {e}"),
            Ok(HostEvent::Button(e)) => broadcast_event(e),
            Ok(HostEvent::Timeout) => wake_and_clean_expired_timers(),
            Ok(HostEvent::Audio(event)) => {
                AUDIO_SENDER.get().unwrap().send(event).unwrap();
            }
            _ => {
                // Anything else means the host disconnected, which should exit the activity.
                std::process::exit(0);
            }
        }
    }
}
