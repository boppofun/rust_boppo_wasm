mod audio_event;

#[cfg(feature = "wasm_client")]
mod audio_handle;

pub use audio_event::AudioEvent;

#[cfg(feature = "wasm_client")]
pub use audio_handle::{AudioHandle, init};

#[cfg(feature = "wasm_client")]
pub(crate) use audio_handle::AUDIO_SENDER;

#[repr(i32)]
pub enum AudioParameter {
    Pause = 0,
    Volume = 1,
    Speed = 3,
}
