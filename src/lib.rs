mod hal;

use boppo_core::log;
pub use hal::executor::block_on;
pub use hal::timer::sleep;

pub fn init() {
    hal::init();
    log::info!("Initiated WASM Activity.");
}
