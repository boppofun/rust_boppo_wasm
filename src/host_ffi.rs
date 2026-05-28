pub mod audio;

#[cfg(feature = "wasm_client")]
pub mod buttons;

pub mod host_event;
#[cfg(feature = "wasm_client")]
pub mod lights;

#[cfg(feature = "wasm_client")]
#[link(wasm_import_module = "host")]
unsafe extern "C" {
    /// Notifies that the wasm file is ready to start
    /// Used in test contexts.
    #[doc(hidden)]
    pub fn boppo_ready();
}

#[cfg(feature = "wasm_client")]
#[doc(hidden)]
pub fn ready() {
    unsafe { boppo_ready() };
}

#[cfg(feature = "wasm_client")]
pub fn init() {
    lights::init_lights();
    buttons::init();
    audio::init();
}
