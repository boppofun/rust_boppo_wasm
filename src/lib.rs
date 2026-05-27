#[cfg(feature = "wasm_client")]
pub mod executor;
#[cfg(feature = "wasm_client")]
pub mod logger;

mod error;
mod host_ffi;
#[cfg(feature = "wasm_client")]
mod timer;

pub use boppo_core::*;
pub use error::Error;
#[cfg(feature = "wasm_client")]
pub use executor::internal_block_on;
#[cfg(feature = "wasm_client")]
pub use host_ffi::audio::AudioHandle;
pub use host_ffi::audio::{self, AudioParameter};
pub use host_ffi::host_event::HostEvent;

#[cfg(feature = "wasm_client")]
pub fn init() {
    executor::init();
    logger::init();
    host_ffi::init();
}
