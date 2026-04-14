mod hal;

pub fn init() {
    hal::init();
}

pub use hal::executor::block_on;
