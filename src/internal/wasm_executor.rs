use std::{
    pin::pin,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use boppo_core::log;
use edge_executor::LocalExecutor;

use crate::{
    audio::OPENED_AUDIO_MAP,
    internal::{HostEvent, buttons::broadcast_event},
};

use crate::internal::timer::{next_timeout, wake_and_clean_expired_timers};

const MAX_TASKS: usize = 32;
static EXECUTOR: AtomicPtr<LocalExecutor<'static, MAX_TASKS>> =
    AtomicPtr::new(std::ptr::null_mut());

/// Initializes the executor.
pub fn init() {
    let executor = Box::leak(Box::new(LocalExecutor::<MAX_TASKS>::new()));
    EXECUTOR.store(executor as *mut _, std::sync::atomic::Ordering::Relaxed);
    boppo_core::hal::set_executor(executor);
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
    static VTABLE: RawWakerVTable = RawWakerVTable::new(
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

#[link(wasm_import_module = "host")]
unsafe extern "C" {
    /// Polling function for Button events with optional timeout.
    /// If timeout_ms < 0, poll will happen indefinitely.
    /// This can be used to poll for button events or wait a certain time if not event was received
    /// in between.
    /// Returns a HostEvent i64 representation.
    pub fn boppo_poll(timeout_ms: i32) -> i64;
}

/// Block on a future with a custom async executor that integrates with the Boppo WASM host.
///
/// Intended to be used to launch an async function from the main (sync) function of the module.
///
/// This function should not be called within a future that is already running.
pub fn block_on<T>(fut: impl Future<Output = T>) -> T {
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
            continue;
        } else {
            next_timeout()
        };
        let raw: Result<HostEvent, u8> = unsafe { boppo_poll(timeout) }.try_into();
        match raw {
            Err(e) => log::debug!("skipping unknown host event: {e}"),
            Ok(HostEvent::Button(e)) => broadcast_event(e),
            Ok(HostEvent::Timeout) => wake_and_clean_expired_timers(),
            Ok(HostEvent::FinishedAudio(handle)) => {
                let mut optional_sender = {
                    let mut map = OPENED_AUDIO_MAP.get().unwrap().write().unwrap();
                    map.remove(&handle)
                };
                if let Some(mut optional_sender) = optional_sender.take()
                    && let Some(sender) = optional_sender.take()
                {
                    let _ = sender.send(());
                }
            }
            Ok(HostEvent::Exit) => {
                // Host requested exit.
                std::process::exit(0);
            }
        }
    }
}
