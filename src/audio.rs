mod controller;
mod host_ffi;

pub mod sound_builder;

pub use controller::Controller;
pub use sound_builder::{ControllerOpts, SoundBuilder};

use crate::{Error, audio::host_ffi::boppo_play_sound_instruction};
use std::{
    collections::BTreeMap,
    sync::{OnceLock, RwLock},
};
use tokio::sync::oneshot::Sender;

pub(crate) static OPENED_AUDIO_MAP: OnceLock<RwLock<BTreeMap<u64, Option<Sender<()>>>>> =
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
    let data = serde_json::to_string(sound.into().as_instruction()).unwrap();
    Error::result_from_neg_i32(unsafe { boppo_play_sound_instruction(data.as_ptr(), data.len()) })?;
    Ok(())
}

/// Wrap `sound` with a controller and play it.
pub fn play_with_controller(sound: impl Into<SoundBuilder>) -> Result<Controller, Error> {
    let (sound, controller) = sound.into().controller();
    play(sound)?;
    Ok(controller)
}

pub async fn play_and_wait_until_finished(sound: impl Into<SoundBuilder>) -> Result<(), Error> {
    play_with_controller(sound)?.wait_until_finished().await;
    Ok(())
}

pub fn stop_all() {
    unsafe {
        host_ffi::boppo_stop_all_sounds();
    }
}

pub fn init() {
    use std::sync::RwLock;

    let _ = OPENED_AUDIO_MAP.set(RwLock::new(BTreeMap::new()));
}
