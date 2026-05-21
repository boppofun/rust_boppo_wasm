use std::{
    collections::BTreeMap,
    f32,
    sync::{OnceLock, RwLock},
};

use boppo_core::log;
use tokio::sync::oneshot::{self, Sender};

#[cfg(feature = "wasm_client")]
use crate::AudioError;
use crate::host_ffi::audio::AudioParameter;

pub(crate) static OPENED_AUDIO_MAP: OnceLock<RwLock<BTreeMap<i32, Option<Sender<()>>>>> =
    OnceLock::new();

#[cfg(feature = "wasm_client")]
#[link(wasm_import_module = "host")]
unsafe extern "C" {
    /// Opens a sound file on the host
    /// Returns an integer ID for the matching sound.
    fn boppo_open_audio_file(path_ptr: *const u8, str_length: usize) -> i32;

    /// Plays an open sound based on its ID.
    fn boppo_play_audio(audio_handle: i32) -> i32;

    /// Sets a parameter to control the sound : volume, pause, etc.
    fn boppo_set_audio_parameter(sound_id: i32, parameter: i32, value: f32) -> i32;

    fn boppo_stop_and_unload_audio(sound_id: i32) -> i32;

}

/// Represents a detached playing audio file that might have already been unloaded.
#[cfg(feature = "wasm_client")]
pub struct AudioHandle(i32);

#[cfg(feature = "wasm_client")]
pub fn init() {
    use std::sync::RwLock;

    let _ = OPENED_AUDIO_MAP.set(RwLock::new(BTreeMap::new()));
}

#[cfg(feature = "wasm_client")]
impl AudioHandle {
    pub fn open(path: &str) -> Result<Self, AudioError> {
        let handle = unsafe { boppo_open_audio_file(path.as_ptr(), path.len()) };
        if handle < 0 {
            Err((-handle).into())
        } else {
            let mut map = OPENED_AUDIO_MAP.get().unwrap().write().unwrap();
            map.insert(handle, None);
            Ok(Self(handle))
        }
    }

    pub fn play(&self) -> Result<(), AudioError> {
        if !self.is_finished() {
            unsafe {
                match boppo_play_audio(self.0) {
                    0 => Ok(()),
                    _ => Err(AudioError::InvalidHandle),
                }
            }
        } else {
            Err(AudioError::InvalidHandle)
        }
    }

    pub fn is_finished(&self) -> bool {
        let map = OPENED_AUDIO_MAP.get().unwrap().read().unwrap();
        map.get(&self.0).is_none()
    }

    pub async fn notify(self) {
        if self.is_finished() {
            return;
        }
        let receiver = {
            let mut map = OPENED_AUDIO_MAP.get().unwrap().write().unwrap();
            let (sender, receiver) = oneshot::channel();
            map.insert(self.0, Some(sender));
            receiver
        };
        if let Err(e) = receiver.await {
            log::error!("Error receiving audio end notifier : {e}");
            // Instead of exposing an internal error to the user, just exist the activity.
            std::process::exit(1);
        }
    }

    pub async fn play_and_notify(self) -> Result<(), AudioError> {
        self.play()?;
        self.notify().await;
        Ok(())
    }

    pub fn set_paused(&self, paused: bool) -> Result<(), AudioError> {
        if self.is_finished() {
            return Err(AudioError::InvalidHandle);
        }
        unsafe {
            if boppo_set_audio_parameter(
                self.0,
                AudioParameter::Pause as i32,
                if paused { 1. } else { 0. },
            ) >= 0
            {
                Ok(())
            } else {
                Err(AudioError::InvalidHandle)
            }
        }
    }

    pub fn set_volume(&self, volume: f32) -> Result<(), AudioError> {
        if self.is_finished() {
            return Err(AudioError::InvalidHandle);
        }
        unsafe {
            if boppo_set_audio_parameter(self.0, AudioParameter::Volume as i32, volume) >= 0 {
                Ok(())
            } else {
                Err(AudioError::InvalidHandle)
            }
        }
    }

    pub fn set_speed(&self, speed: f32) -> Result<(), AudioError> {
        if self.is_finished() {
            return Err(AudioError::InvalidHandle);
        }
        unsafe {
            if boppo_set_audio_parameter(self.0, AudioParameter::Speed as i32, speed) >= 0 {
                Ok(())
            } else {
                Err(AudioError::InvalidHandle)
            }
        }
    }

    // This does not return an error because stopping an already stopped audio shouldn't
    // do anything, thus can't fail.
    pub fn stop(self) {
        if self.is_finished() {
            return;
        }
        unsafe {
            boppo_stop_and_unload_audio(self.0);
        }
    }
}
