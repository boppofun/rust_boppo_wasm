//! Items internal to the WASM framework or used to initialize it.
//!
//! Activity developers should not need to use these items directly
//! unless they are manually initializing the WASM framework instead
//! of using the default `init_and_run_async`.
//!
//! Many items in internal are exported for the WASM host.
pub mod audio;

#[cfg(feature = "wasm_client")]
mod buttons;
mod host_event;
#[cfg(feature = "wasm_client")]
pub(crate) mod host_ffi;
#[cfg(feature = "wasm_client")]
mod lights;
#[cfg(feature = "wasm_client")]
mod logger;
#[cfg(feature = "wasm_client")]
mod timer;
#[cfg(feature = "wasm_client")]
mod wasm_executor;

pub use host_event::HostEvent;
#[cfg(feature = "wasm_client")]
pub use wasm_executor::block_on;

/// Initialize all WASM subsystems (logger, lights, buttons, executor, audio).
///
/// Called automatically by [`crate::init_and_run_async`]. Only call this directly
/// if you are manually driving the runtime instead of using that helper.
#[cfg(feature = "wasm_client")]
pub fn init() {
    logger::init();
    lights::init_lights();
    buttons::init();
    wasm_executor::init();
    crate::audio::init();
}
