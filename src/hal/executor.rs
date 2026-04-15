//! Since edge executor is scheduled to be removed from boppo_core,
//! We should provide it with an async executor in WASM.
//! For now, it mimics what happens in boppo_core.

use std::{
    pin::pin,
    sync::atomic::AtomicPtr,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use edge_executor::LocalExecutor;

use crate::hal::buttons::{boppo_wasm_poll, register_event};

const MAX_TASKS: usize = 32;
static EXECUTOR: AtomicPtr<LocalExecutor<'static, MAX_TASKS>> =
    AtomicPtr::new(std::ptr::null_mut());

pub fn init() {
    let executor = Box::leak(Box::new(LocalExecutor::<MAX_TASKS>::new()));
    EXECUTOR.store(executor as *mut _, std::sync::atomic::Ordering::SeqCst);
    boppo_core::hal::set_executor(executor);
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
/// with its own executor, we just need a dummy no-op waker to satisfy the API.
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
        // Poll actual events from the host and register events so that
        // futures polled above can yield below
        let raw = unsafe {
            // TODO: insert next timer here
            boppo_wasm_poll(0)
        };
        register_event(raw);

        // Poll futures - the actual waiting happens below
        if let Poll::Ready(v) = top.as_mut().poll(&mut cx) {
            return v;
        }
    }
}
