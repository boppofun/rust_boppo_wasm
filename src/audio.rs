mod controller;
mod host_ffi;

pub mod sound_builder;

use boppo_core::log::error;
pub use controller::Controller;
pub use sound_builder::{ControllerOpts, SoundBuilder};

use crate::{Error, audio::host_ffi::boppo_play_sound_instruction};
use std::{
    collections::BTreeMap,
    sync::{OnceLock, RwLock},
};
use tokio::sync::oneshot::Sender;

/// An empty vec signifies that the sound is currently playing but has no controller
/// that is waiting for it to finish.
pub(crate) static PLAYING_CONTROLLERS: OnceLock<RwLock<BTreeMap<u64, Vec<Sender<()>>>>> =
    OnceLock::new();

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
    Error::result_from_neg_i32(unsafe { boppo_play_sound_instruction(data.as_ptr(), data.len()) })?;
    // Now that we know the sound is playing insert an empty vec to signify that
    // the sound is playing but has no controller yet
    for id in ids {
        PLAYING_CONTROLLERS
            .get()
            .unwrap()
            .write()
            .unwrap()
            .entry(id)
            .or_default();
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
        // the host is responsible for sending finished notifications that will clear out PLAYING_CONTROLLERS
        host_ffi::boppo_stop_all_sounds();
    }
}

pub(crate) fn init() {
    use std::sync::RwLock;

    let _ = PLAYING_CONTROLLERS.set(RwLock::new(BTreeMap::new()));
}
