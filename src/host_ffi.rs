pub mod audio;

#[cfg(feature = "wasm_client")]
pub mod buttons;

pub mod host_event;
#[cfg(feature = "wasm_client")]
pub mod lights;

#[cfg(feature = "wasm_client")]
pub fn init() {
    lights::init_lights();
    buttons::init();
    audio::init();
}
