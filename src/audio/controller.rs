use crate::audio::OPENED_AUDIO_MAP;
use tokio::sync::oneshot;

use crate::Error;
use crate::internal::audio::AudioParameter;

/// A controller for a sound actively playing.
pub struct Controller(i32);

impl Controller {
    /// Return `true` if the sound has finished playing or has been stopped.
    pub fn is_finished(&self) -> bool {
        let map = OPENED_AUDIO_MAP.get().unwrap().read().unwrap();
        map.get(&self.0).is_none()
    }

    /// Wait until the sound has finished playing or has been stopped.
    pub async fn wait_until_finished(self) {
        if self.is_finished() {
            return;
        }
        // Single threaded reliance: Its fine to check if finished and then insert the notifier since
        // we only receive notifications when we poll and our WASM executor is single threaded.
        let receiver = {
            let mut map = OPENED_AUDIO_MAP.get().unwrap().write().unwrap();
            let (sender, receiver) = oneshot::channel();
            map.insert(self.0, Some(sender));
            receiver
        };
        let _ = receiver.await;
    }

    /// Pause or unpause the sound.
    ///
    /// `paused` is `true` if the sound should be paused, `false` if it should be unpaused.
    pub fn set_paused(&self, paused: bool) -> Result<(), Error> {
        let value = if paused { 1. } else { 0. };
        self.set_controller_parameter(AudioParameter::Pause, value)
    }

    /// Set the volume of the sound.
    ///
    /// The samples are multiplied by `multiplier` so 1.0 would leave the Sound
    /// unchanged. 0.5 would reduce the sample values by half and 2.0 would
    /// double them (saturating if larger than the max value).
    pub fn set_volume(&self, multiplier: f32) -> Result<(), Error> {
        self.set_controller_parameter(AudioParameter::Volume, multiplier)
    }

    pub fn set_speed(&self, multiplier: f32) -> Result<(), Error> {
        self.set_controller_parameter(AudioParameter::Speed, multiplier)
    }

    pub fn stop(self) -> Result<(), Error> {
        self.set_controller_parameter(AudioParameter::Stop, 1.0)
    }

    fn set_controller_parameter(&self, param: AudioParameter, value: f32) -> Result<(), Error> {
        unsafe {
            Error::result_from_neg_i32(super::host_ffi::boppo_set_controller_parameter(
                self.0,
                param as i32,
                value,
            ))?;
        };
        Ok(())
    }
}
