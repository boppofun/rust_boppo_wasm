#[cfg(feature = "wasm_client")]
mod audio_handle;

use std::{error::Error, fmt::Display};

#[cfg(feature = "wasm_client")]
pub use audio_handle::{AudioHandle, init, stop_all};

#[cfg(feature = "wasm_client")]
pub(crate) use audio_handle::OPENED_AUDIO_MAP;

#[repr(i32)]
pub enum AudioParameter {
    Pause = 0,
    Volume = 1,
    Speed = 2,
}

#[repr(i32)]
#[derive(Debug)]
pub enum AudioError {
    InvalidHandle = 1,
    NotFound = 2,
    PermissionDenied = 3,
    InvalidPath = 4,
    InvalidParameter = 5,
    Unknown = 255,
}

impl Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::InvalidHandle => f.write_str("Invalid audio handle."),
            AudioError::InvalidPath => f.write_str("Invalid file path"),
            AudioError::PermissionDenied => f.write_str("Permission denied"),
            AudioError::NotFound => f.write_str("File not found"),
            AudioError::InvalidParameter => f.write_str("Invalid audio parameter"),
            AudioError::Unknown => f.write_str("Unknown audio error"),
        }
    }
}

impl Error for AudioError {}

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

impl From<i32> for AudioError {
    fn from(value: i32) -> Self {
        match value {
            1 => AudioError::InvalidHandle,
            2 => AudioError::NotFound,
            3 => AudioError::PermissionDenied,
            4 => AudioError::InvalidPath,
            5 => AudioError::InvalidParameter,
            _ => AudioError::Unknown,
        }
    }
}
