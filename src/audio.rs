mod controller;
mod host_ffi;

use std::{
    collections::BTreeMap,
    sync::{OnceLock, RwLock},
};

pub use controller::Controller;
use tokio::sync::oneshot::Sender;

use crate::{Error, audio::host_ffi::boppo_play_sound_instruction};

pub(crate) static OPENED_AUDIO_MAP: OnceLock<RwLock<BTreeMap<i32, Option<Sender<()>>>>> =
    OnceLock::new();

// TODO change this to take a SoundBuilder
pub fn play(path: &str) -> Result<(), Error> {
    let handle = Error::result_from_neg_i32(unsafe {
        boppo_play_sound_instruction(path.as_ptr(), path.len())
    })?;
    let mut map = OPENED_AUDIO_MAP.get().unwrap().write().unwrap();
    map.insert(handle, None);
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
