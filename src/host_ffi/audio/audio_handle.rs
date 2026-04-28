use std::sync::OnceLock;

use super::AudioEvent;
use boppo_core::log;
use tokio::sync::{broadcast, broadcast::Receiver};

pub(crate) static AUDIO_SENDER: OnceLock<broadcast::Sender<AudioEvent>> = OnceLock::new();

#[cfg(feature = "wasm_client")]
#[link(wasm_import_module = "host")]
unsafe extern "C" {
    /// Opens a sound file on the host
    /// Returns an integer ID for the matching sound.
    fn open_sound_file(path_ptr: *const u8, str_length: usize) -> i32;

    /// Plays an open sound based on its ID.
    fn play_sound(sound_id: i32);

    /// Sets a parameter to control the sound : volume, pause, etc.
    fn set_sound_parameter(sound_id: i32, parameter: i32, value: f32);

    fn stop_sound(sound_id: i32);

    fn unload_sound_file(sound_id: i32);
}

#[cfg(feature = "wasm_client")]
pub struct AudioHandle(i32, Receiver<AudioEvent>);

#[cfg(feature = "wasm_client")]
#[repr(i32)]
pub enum SoundParameter {
    Pause = 0,
    Volume = 1,
    Speed = 3,
}

#[cfg(feature = "wasm_client")]
pub fn init() {
    let (sender, _) = broadcast::channel(16);
    let _ = AUDIO_SENDER.set(sender);
}

#[cfg(feature = "wasm_client")]
impl AudioHandle {
    pub fn open(path: &str) -> Result<Self, ()> {
        let handle = unsafe { open_sound_file(path.as_ptr(), path.len()) };
        if handle < 0 {
            Err(())
        } else {
            Ok(Self(handle, AUDIO_SENDER.get().unwrap().subscribe()))
        }
    }

    pub fn play(&self) {
        unsafe { play_sound(self.0) }
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

    pub fn set_paused(&self, paused: bool) {
        todo!()
    }

    pub fn set_volume(&self, volume: f32) {
        todo!()
    }

    pub fn set_speed(&self, speed: f32) {
        todo!()
    }

    pub fn stop(&self) {
        todo!()
    }
}

#[cfg(feature = "wasm_client")]
impl Drop for AudioHandle {
    fn drop(&mut self) {
        unsafe {
            unload_sound_file(self.0);
        }
    }
}
