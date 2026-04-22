pub mod executor;
mod host_ffi;
pub mod logger;
pub mod timer;

pub use executor::block_on;
pub use executor::spawn;
pub use host_ffi::host_event::HostEvent;

pub fn init() {
    executor::init();
    logger::init();
    host_ffi::init();
}
