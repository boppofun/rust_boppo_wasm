use std::sync::OnceLock;

use boppo_core::{ButtonEvent, log};
use tokio::sync::broadcast;

#[link(wasm_import_module = "host")]
unsafe extern "C" {
    /// Polling function for Button events with optionnal timeout.
    /// If timeout_ms <= 0, poll will happen indefinitely.
    /// This can be used to poll for button events or wait a certain time if not event was received
    /// in between.
    /// Returns a ButtonEvent u16 representation (>= 0) on any button event, -1 on timeout, -2 if channel diconnected early.
    pub fn boppo_wasm_poll(timeout_ms: i32) -> i32;
}

static BUTTON_SENDER: OnceLock<broadcast::Sender<ButtonEvent>> = OnceLock::new();

/// Registers an event to an event queue when a button event is sent from the host
///
/// The raw_wasm_code is an i32 representing a ButtonEvent if it's >= 0, a timeout if
/// it's equal to -1, and a closed channel if it's equal to -2 that should exit early.
pub fn register_event(raw_wasm_code: i32) {
    match raw_wasm_code {
        e if e >= 0 => {
            let event = ButtonEvent::from_u16(raw_wasm_code as u16);
            let _ = BUTTON_SENDER.get().unwrap().send(event);
        }
        -1 => {
            //timeout has been reached
            // TODO : wake the next waiting timer
            todo!()
        }
        _ => {
            log::info!("WASM received close signal. Exiting...");
            // channel closed (-2 or any other).
            // This means the top-level activity thread is requesting a clean exit.
            std::process::exit(0);
        }
    }
}

pub fn init() {
    let (sender, _) = broadcast::channel::<ButtonEvent>(16);
    BUTTON_SENDER.set(sender.clone()).ok();
    boppo_core::hal::set_button_events(sender);
}
