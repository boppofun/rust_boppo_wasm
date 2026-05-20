#[cfg(feature = "wasm_client")]
mod audio_handle;

#[cfg(feature = "wasm_client")]
pub use audio_handle::{AudioHandle, init};

#[cfg(feature = "wasm_client")]
pub(crate) use audio_handle::OPENED_AUDIO_MAP;

#[repr(i32)]
pub enum AudioParameter {
    Pause = 0,
    Volume = 1,
    Speed = 2,
}

impl TryFrom<i32> for AudioParameter {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Pause),
            1 => Ok(Self::Volume),
            2 => Ok(Self::Speed),
            _ => Err("Unknown audio parameter code."),
        }
    }
}
