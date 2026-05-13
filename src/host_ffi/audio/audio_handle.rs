use std::{error::Error, f32, fmt::Display, sync::OnceLock};

use super::AudioEvent;
use boppo_core::log;
use tokio::sync::{broadcast, broadcast::Receiver};

use crate::host_ffi::audio::AudioParameter;

pub(crate) static AUDIO_SENDER: OnceLock<broadcast::Sender<AudioEvent>> = OnceLock::new();

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

    fn boppo_stop_audio(sound_id: i32);

    fn boppo_unload_audio(sound_id: i32);
}

/// Represents an audio file
// This is meant to hold "distant ownership" of an audio file
// on the host thread. Dropping it will clear the audio file on
// the host side. The interface is made so that playing or awaiting
// completion of this struct effectively drops it, triggering clean-up on the host
// so that entries don't pile up.
#[cfg(feature = "wasm_client")]
pub struct AudioHandle(i32, Receiver<AudioEvent>);

/// Represents a detached playing audio file that might have already been unloaded.
#[cfg(feature = "wasm_client")]
pub struct DetachedAudioHandle(Option<AudioHandle>);

#[cfg(feature = "wasm_client")]
pub fn init() {
    let (sender, _) = broadcast::channel(16);
    let _ = AUDIO_SENDER.set(sender);
}

#[cfg(feature = "wasm_client")]
impl AudioHandle {
    pub fn open(path: &str) -> Result<Self, ()> {
        let handle = unsafe { boppo_open_audio_file(path.as_ptr(), path.len()) };
        if handle < 0 {
            Err(())
        } else {
            let event_receiver = AUDIO_SENDER.get().unwrap().subscribe();
            Ok(Self(handle, event_receiver))
        }
    }

    pub fn play(self) -> DetachedAudioHandle {
        unsafe {
            boppo_play_audio(self.0);
        }
        DetachedAudioHandle(Some(self))
    }

    async fn internal_play_and_notify(&mut self) -> Result<(), BadHandleError> {
        unsafe {
            boppo_play_audio(self.0);
        }
        loop {
            let event = self.1.recv().await;
            match event {
                Ok(AudioEvent::Finished(handle)) => {
                    if handle == self.0 {
                        break Ok(());
                    }
                }
                Ok(AudioEvent::BadHandleError(handle)) => {
                    if handle == self.0 {
                        break Err(BadHandleError);
                    }
                }
                Err(e) => {
                    // If we do our job correctly, this should never happen
                    log::error!("Error receiving audio event : {e}");

                    // Instead of exposing an internal error to the user, just exist the activity.
                    std::process::exit(1);
                }
            }
        }
    }

    pub async fn play_and_notify(mut self) {
        let _ = self.internal_play_and_notify().await;
    }

    fn set_paused(&self, paused: bool) {
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
            boppo_stop_audio(self.0);
        }
    }
}

#[cfg(feature = "wasm_client")]
impl Drop for AudioHandle {
    fn drop(&mut self) {
        unsafe {
            boppo_unload_audio(self.0);
        }
    }
}

impl DetachedAudioHandle {
    // Wait for this audio to finish. If it is already finished, it resolves immediately.
    pub async fn notify(mut self) {
        // Taking the internal handle ensures it is dropped at the end of this function
        if let Some(mut handle) = self.0.take() {
            // Wether the future resolves with a Finished or BadHandleError, it means playback is over.
            let _ = handle.internal_play_and_notify().await;
        }
        //Handle already consumed. Do nothing.
    }

    pub fn try_set_paused(&mut self, paused: bool) -> Result<(), BadHandleError> {
        if let Some(handle) = &mut self.0 {
            handle.set_paused(paused);
            Ok(())
        } else {
            Err(BadHandleError)
        }
    }

    pub fn try_set_volume(&mut self, value: f32) -> Result<(), BadHandleError> {
        if let Some(handle) = &mut self.0 {
            handle.set_volume(value);
            Ok(())
        } else {
            Err(BadHandleError)
        }
    }

    pub fn try_set_speed(&mut self, value: f32) -> Result<(), BadHandleError> {
        if let Some(handle) = &mut self.0 {
            handle.set_speed(value);
            Ok(())
        } else {
            Err(BadHandleError)
        }
    }

    pub fn stop(mut self) {
        if let Some(handle) = self.0.take() {
            handle.stop();
        }
    }
}
