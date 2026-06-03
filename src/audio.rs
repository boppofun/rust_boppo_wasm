//! Audio playback and control for Boppo activities.
pub use boppo_core::audio::Controller;
pub use boppo_core::audio::*;
pub use boppo_core::audio::{ControllerOpts, SoundBuilder};
use boppo_core::log::error;

use crate::{Error, internal::host_ffi};

/// Play `sound`
///
/// ## Examples
///
/// Play a file:
///
/// ```rust,no_run
/// # use boppo_wasm::audio::play;
/// play("music.mp3")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn play(sound: impl Into<SoundBuilder>) -> Result<(), Error> {
    let si = sound.into();
    let data = match serde_json::to_string(si.as_instruction()) {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to serialize sound instruction: {e:?}");
            return Err(Error::InvalidParameter);
        }
    };
    let Some(ids) = si.as_instruction().controller_ids() else {
        error!("Controller found inside Repeat.");
        return Err(Error::InvalidParameter);
    };
    Error::result_from_i32(unsafe {
        host_ffi::boppo_play_sound_instruction(data.as_ptr(), data.len())
    })?;
    // Now that we know the sound is playing insert an empty vec to signify that
    // the sound is playing but has no controller yet
    for id in ids {
        boppo_core::hal::set_sound_controller_as_playing(id);
    }
    Ok(())
}

/// Wrap `sound` with a controller and play it.
///
/// This is a convenience wrapper around [`SoundBuilder::controller`] and [`play`]
pub fn play_with_controller(sound: impl Into<SoundBuilder>) -> Result<Controller, Error> {
    let (sound, controller) = sound.into().controller();
    play(sound)?;
    Ok(controller)
}

/// Play `sound` and wait until it finishes.
///
/// This is a convenience wrapper around [`SoundBuilder::controller`], [`play`], and [`Controller::wait_until_finished`].
pub async fn play_and_wait_until_finished(sound: impl Into<SoundBuilder>) -> Result<(), Error> {
    play_with_controller(sound)?.wait_until_finished().await;
    Ok(())
}

/// Stop all currently playing sounds.
///
/// This includes stopping sounds that are not controlled by a [`Controller`].
///
/// Sounds controlled by a [`Controller`] will receive a finished notification.
pub fn stop_all() {
    unsafe {
        // the host is responsible for sending finished notifications that will clear out
        // the controllers map in boppo_core
        host_ffi::boppo_stop_all_sounds();
    }
}

pub(crate) fn init() {
    boppo_core::hal::init_audio(set_controller_parameter);
}

fn set_controller_parameter(id: u64, param: boppo_core::hal::AudioParameter, value: f32) {
    let result = unsafe { host_ffi::boppo_set_controller_parameter(id, param as i32, value) };
    match Error::result_from_i32(result) {
        Ok(_) => (),
        Err(Error::NotFound) => {
            // Sound might have just finished already which should not be considered an error.
        }
        Err(e) => {
            // Parameters have been validated already so we should not see any other errors.
            panic!("Unexpected error setting controller parameter: {:?}", e);
        }
    }
}
