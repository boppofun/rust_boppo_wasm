pub mod buttons;
pub mod executor;
pub mod lights;
pub mod logger;

pub fn init() {
    logger::init();
    executor::init();
    lights::init_lights();
    buttons::init();
}
