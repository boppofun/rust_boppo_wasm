use std::{
    collections::BTreeMap,
    error::Error,
    f32,
    fmt::Display,
    sync::{Mutex, OnceLock},
};

use boppo_core::log;
use tokio::sync::oneshot::{self, Receiver, Sender};

use crate::host_ffi::audio::AudioParameter;

pub(crate) static OPENED_AUDIO_MAP: OnceLock<Mutex<BTreeMap<i32, Option<Sender<()>>>>> =
    OnceLock::new();

#[derive(Debug)]
pub struct BadHandleError;

impl Display for BadHandleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Incorrect Audio Handle.")
    }
}

impl Error for BadHandleError {}

#[cfg(feature = "wasm_client")]
#[link(wasm_import_module = "host")]
unsafe extern "C" {
    /// Opens a sound file on the host
    /// Returns an integer ID for the matching sound.
    fn boppo_open_audio_file(path_ptr: *const u8, str_length: usize) -> i32;

    /// Plays an open sound based on its ID.
    fn boppo_play_audio(audio_handle: i32);

    /// Sets a parameter to control the sound : volume, pause, etc.
    fn boppo_set_audio_parameter(sound_id: i32, parameter: i32, value: f32);

    fn boppo_stop_and_unload_audio(sound_id: i32);

}

/// Represents a detached playing audio file that might have already been unloaded.
#[cfg(feature = "wasm_client")]
pub struct AudioHandle(i32, Option<Receiver<()>>);

#[cfg(feature = "wasm_client")]
pub fn init() {
    let _ = OPENED_AUDIO_MAP.set(Mutex::new(BTreeMap::new()));
}

#[cfg(feature = "wasm_client")]
impl AudioHandle {
    pub fn open(path: &str) -> Result<Self, ()> {
        let handle = unsafe { boppo_open_audio_file(path.as_ptr(), path.len()) };
        if handle < 0 {
            Err(())
        } else {
            Ok(Self(handle, None))
        }
    }

    pub fn play(&self) -> Result<(), BadHandleError> {
        if !self.is_finished() {
            unsafe {
                boppo_play_audio(self.0);
            }
            Ok(())
        } else {
            Err(todo!())
        }
    }

    pub fn is_finished(&self) -> bool {
        todo!()
    }

    pub async fn play_and_notify(self) -> Result<(), BadHandleError> {
        self.play()?;
        let (sender, receiver) = oneshot::channel();
        OPENED_AUDIO_MAP
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .insert(self.0, Some(sender));
        match receiver.await {
            Ok(()) => Ok(()),
            Err(e) => {
                // If we do our job correctly, this should never happen
                log::error!("Error receiving audio event : {e}");

                // Instead of exposing an internal error to the user, just exist the activity.
                std::process::exit(1);
            }
        }
    }

    pub fn set_paused(&self, paused: bool) {
        unsafe {
            boppo_set_audio_parameter(
                self.0,
                AudioParameter::Pause as i32,
                if paused { 1. } else { 0. },
            );
        }
    }

    pub fn set_volume(&self, volume: f32) {
        unsafe {
            boppo_set_audio_parameter(self.0, AudioParameter::Volume as i32, volume);
        }
    }

    pub fn set_speed(&self, speed: f32) {
        unsafe {
            boppo_set_audio_parameter(self.0, AudioParameter::Speed as i32, speed);
        }
    }

    pub fn stop(self) {
        unsafe {
            boppo_stop_and_unload_audio(self.0);
        }
    }
}

#[cfg(feature = "wasm_client")]
impl Drop for AudioHandle {
    fn drop(&mut self) {
        unsafe {
            boppo_stop_and_unload_audio(self.0);
        }
    }
}
