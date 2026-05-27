mod audio_handle;

use std::{
    collections::BTreeMap,
    sync::{OnceLock, RwLock},
};

pub use audio_handle::AudioHandle;
use tokio::sync::oneshot::Sender;

pub(crate) static OPENED_AUDIO_MAP: OnceLock<RwLock<BTreeMap<i32, Option<Sender<()>>>>> =
    OnceLock::new();

#[link(wasm_import_module = "host")]
unsafe extern "C" {
    /// Stops and unloads all currently loaded audio clips.
    fn boppo_stop_all_audio();
}

/// Stops and unloads all currently loaded audio clips, invalidating all
/// existing audio handles, even unplayed ones.
pub fn stop_all() {
    unsafe {
        boppo_stop_all_audio();
    }
}

pub fn init() {
    use std::sync::RwLock;

    let _ = OPENED_AUDIO_MAP.set(RwLock::new(BTreeMap::new()));
}
