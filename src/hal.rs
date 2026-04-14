pub mod buttons;
pub mod executor;
pub mod lights;

pub fn init() {
    executor::init();
    lights::init_lights();
    buttons::init();
}
