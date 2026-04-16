pub mod executor;
mod host_ffi;
pub mod logger;
pub mod timer;

use boppo_core::log;
pub use executor::block_on;
pub use executor::spawn;
pub use timer::sleep;

pub fn init() {
    executor::init();
    logger::init();
    host_ffi::init();
    log::info!("Initiated WASM Activity.");
}
