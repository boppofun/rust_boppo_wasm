mod hal;

pub fn init() {
    hal::init();
    log::info!("Initiated WASM Activity.");
}

use boppo_core::log;
pub use hal::executor::block_on;
