use std::{
    collections::BTreeMap,
    f32,
    sync::{OnceLock, RwLock},
};

use tokio::sync::oneshot::{self, Sender};

#[cfg(feature = "wasm_client")]
use crate::BoppoError;
use crate::host_ffi::audio::AudioParameter;

pub(crate) static OPENED_AUDIO_MAP: OnceLock<RwLock<BTreeMap<i32, Option<Sender<()>>>>> =
    OnceLock::new();

#[cfg(feature = "wasm_client")]
#[link(wasm_import_module = "host")]
unsafe extern "C" {
    /// Opens an audio file for playing.
    /// Returns a positive integer representing the audio handle, and a negative error code otherwise.
    ///
    /// ## Failure
    ///
    /// Returns a negative handle in case of failure:
    /// * -2 if the file was not found
    /// * -3 if the requested file path is restricted (permission denied)
    /// * -4 if the path itself is invalid
    /// * -255 for internal errors
    fn boppo_open_audio_file(path_ptr: *const u8, str_length: usize) -> i32;

    /// Plays an opened audio file using its handle.
    /// Returns 0 if successful, -1 if handle was invalid.
    fn boppo_play_audio(audio_handle: i32) -> i32;

    ///Sets an audio parameter for an opened audio clip, using its handle,
    /// a parameter code, and an f32 value:
    /// * Parameter code 0 : pauses (1.) or unpauses (0.) a clip.
    /// * Parameter code 1 : Volume, with values ranging from 0. to 1.
    /// * Parameter code 2 : Speed multiplier with values ranging from 0. to 1.
    ///
    /// Returns 0 if successful, -1 if the provided handle was invalid, and -5 if the parameter
    /// code was unrecognized.
    fn boppo_set_audio_parameter(sound_id: i32, parameter: i32, value: f32) -> i32;

    /// Stops and unloads an opened audio clip based on its handle.
    /// Returns 0 if successful, -1 if handle was invalid.
    fn boppo_stop_audio(sound_id: i32) -> i32;

    /// Stops and unloads all currently loaded audio clips.
    fn boppo_stop_all_audio();

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
    pub fn open(path: &str) -> Result<Self, BoppoError> {
        let handle =
            unsafe { status_code_to_result(boppo_open_audio_file(path.as_ptr(), path.len()))? };
        let mut map = OPENED_AUDIO_MAP.get().unwrap().write().unwrap();
        map.insert(handle, None);
        Ok(Self(handle))
    }

    pub fn play(&self) -> Result<(), BoppoError> {
        if !self.is_finished() {
            unsafe {
                status_code_to_result(boppo_play_audio(self.0))?;
            }
            Ok(())
        } else {
            Err(BoppoError::NotFound)
        }
    }

    pub fn is_finished(&self) -> bool {
        let map = OPENED_AUDIO_MAP.get().unwrap().read().unwrap();
        map.get(&self.0).is_none()
    }

    pub async fn wait_until_finished(self) {
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
            // Instead of exposing an internal error to the user, just panic & exit the activity.
            panic!("Error receiving audio end notifier : {e}");
        }
    }

    pub async fn play_and_wait_until_finished(self) -> Result<(), BoppoError> {
        self.play()?;
        self.wait_until_finished().await;
        Ok(())
    }

    pub fn set_paused(&self, paused: bool) -> Result<(), BoppoError> {
        unsafe {
            status_code_to_result(boppo_set_audio_parameter(
                self.0,
                AudioParameter::Pause as i32,
                if paused { 1. } else { 0. },
            ))?;
        }
        Ok(())
    }

    pub fn set_volume(&self, volume: f32) -> Result<(), BoppoError> {
        unsafe {
            status_code_to_result(boppo_set_audio_parameter(
                self.0,
                AudioParameter::Volume as i32,
                volume,
            ))?;
        }
        Ok(())
    }

    pub fn set_speed(&self, speed: f32) -> Result<(), BoppoError> {
        unsafe {
            status_code_to_result(boppo_set_audio_parameter(
                self.0,
                AudioParameter::Speed as i32,
                speed,
            ))?;
        }
        Ok(())
    }

    pub fn stop(self) -> Result<(), BoppoError> {
        unsafe {
            status_code_to_result(boppo_stop_audio(self.0))?;
        }
        Ok(())
    }
}

/// Stops and unloads all currently loaded audio clips, invalidating all
/// existing audio handles, even unplayed ones.
pub fn stop_all() {
    unsafe {
        boppo_stop_all_audio();
    }
}

fn status_code_to_result(n: i32) -> Result<i32, BoppoError> {
    if n < 0 {
        Err(BoppoError::from(-n))
    } else {
        Ok(n)
    }
}
