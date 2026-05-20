#[cfg(feature = "wasm_client")]
pub mod executor;

mod host_ffi;

#[cfg(feature = "wasm_client")]
pub mod logger;

#[cfg(feature = "wasm_client")]
pub mod timer;

#[cfg(feature = "wasm_client")]
pub use executor::internal_block_on;

#[cfg(feature = "wasm_client")]
pub use executor::spawn;

pub use host_ffi::audio::AudioParameter;
pub use host_ffi::host_event::HostEvent;

pub use boppo_core;

#[cfg(feature = "wasm_client")]
pub use host_ffi::audio::{AudioHandle, DetachedAudioHandle};

#[cfg(feature = "wasm_client")]
pub fn init() {
    executor::init();
    logger::init();
    host_ffi::init();
}
