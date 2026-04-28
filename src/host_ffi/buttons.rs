use std::sync::OnceLock;

use boppo_core::ButtonEvent;
use tokio::sync::broadcast;

static BUTTON_SENDER: OnceLock<broadcast::Sender<ButtonEvent>> = OnceLock::new();

///Broadcasts a button event to all listeners registered through boppo_core's HAL, on the wasm side.
pub fn broadcast_event(event: ButtonEvent) {
    let _ = BUTTON_SENDER.get().unwrap().send(event);
}

pub fn init() {
    let (sender, _) = broadcast::channel::<ButtonEvent>(16);
    BUTTON_SENDER.set(sender.clone()).ok();
    boppo_core::hal::set_button_events(sender);
}
