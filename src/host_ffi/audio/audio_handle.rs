use std::sync::OnceLock;

use super::AudioEvent;
use boppo_core::log;
use tokio::sync::{broadcast, broadcast::Receiver};

use crate::host_ffi::audio::AudioParameter;

pub(crate) static AUDIO_SENDER: OnceLock<broadcast::Sender<AudioEvent>> = OnceLock::new();

#[cfg(feature = "wasm_client")]
#[link(wasm_import_module = "host")]
unsafe extern "C" {
    /// Opens a sound file on the host
    /// Returns an integer ID for the matching sound.
    fn boppo_wasm_open_audio_file(path_ptr: *const u8, str_length: usize) -> i32;

    /// Plays an open sound based on its ID.
    fn boppo_wasm_play_audio(audio_handle: i32);
    /*
    /// Sets a parameter to control the sound : volume, pause, etc.
    fn boppo_wasm_set_audio_parameter(sound_id: i32, parameter: i32, value: f32);

    fn boppo_wasm_stop_audio(sound_id: i32);

    fn boppo_wasm_unload_audio_file(sound_id: i32);*/
}

#[cfg(feature = "wasm_client")]
pub struct AudioHandle(i32, Receiver<AudioEvent>);

#[cfg(feature = "wasm_client")]
pub fn init() {
    let (sender, _) = broadcast::channel(16);
    let _ = AUDIO_SENDER.set(sender);
}

#[cfg(feature = "wasm_client")]
impl AudioHandle {
    pub async fn open(path: &str) -> Result<Self, ()> {
        let req = unsafe { boppo_wasm_open_audio_file(path.as_ptr(), path.len()) };
        if req < 0 {
            Err(())
        } else {
            let mut event_receiver = AUDIO_SENDER.get().unwrap().subscribe();
            loop {
                let event = event_receiver.recv().await;
                match event {
                    Ok(AudioEvent::Opened { req_id, handle }) => {
                        if req_id == req {
                            return Ok(Self(handle, event_receiver));
                        }
                    }
                    Err(e) => {
                        log::error!("Error receiving audio event. : {e}");
                        return Err(());
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn play(&self) {
        unsafe { boppo_wasm_play_audio(self.0) }
    }

    pub async fn play_and_notify(&mut self) {
        self.play();
        loop {
            let event = self.1.recv().await;
            match event {
                Ok(AudioEvent::Finished(handle)) => {
                    if handle == self.0 {
                        break;
                    }
                }
                Err(e) => {
                    log::error!("Error receiving audio event. : {e}");
                    break;
                }
                _ => {}
            }
        }
    }

    /*pub fn set_paused(&self, paused: bool) {
        unsafe {
            boppo_wasm_set_audio_parameter(
                self.0,
                AudioParameter::Pause as i32,
                if paused { 1. } else { 0. },
            );
        }
    }

    pub fn set_volume(&self, volume: f32) {
        unsafe {
            boppo_wasm_set_audio_parameter(self.0, AudioParameter::Volume as i32, volume);
        }
    }

    pub fn set_speed(&self, speed: f32) {
        unsafe {
            boppo_wasm_set_audio_parameter(self.0, AudioParameter::Speed as i32, speed);
        }
    }

    pub fn stop(&self) {
        unsafe {
            boppo_wasm_stop_audio(self.0);
        }
    }*/
}

#[cfg(feature = "wasm_client")]
impl Drop for AudioHandle {
    fn drop(&mut self) {
        unsafe {
            //boppo_wasm_unload_sound_file(self.0);
        }
    }
}
