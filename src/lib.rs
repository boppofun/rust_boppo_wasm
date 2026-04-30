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

pub use host_ffi::host_event::HostEvent;

#[cfg(feature = "wasm_client")]
pub fn init() {
    executor::init();
    logger::init();
    host_ffi::init();
}

#[macro_export]
macro_rules! boppo_async_main {
    ($name : ident) => {
        fn main() {
            use boppo_wasm_activity::{init, internal_block_on};
            init();
            internal_block_on($name());
        }
    };
}
