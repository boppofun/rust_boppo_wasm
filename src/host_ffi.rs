pub mod buttons;
pub mod host_event;
pub mod lights;

pub fn init() {
    lights::init_lights();
    buttons::init();
}
