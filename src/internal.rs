//! Items internal to the WASM framework or used it initialize it.
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

#[cfg(feature = "wasm_client")]
pub fn init() {
    logger::init();
    lights::init_lights();
    buttons::init();
    wasm_executor::init();
    crate::audio::init();
}
