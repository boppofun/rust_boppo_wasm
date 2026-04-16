pub mod buttons;
pub mod lights;

pub fn init() {
    lights::init_lights();
    buttons::init();
}
