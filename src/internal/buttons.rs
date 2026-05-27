use std::sync::OnceLock;

use boppo_core::ButtonEvent;
use boppo_core::hal::ButtonCounts;
use tokio::sync::{broadcast, watch};

static BUTTON_SENDER: OnceLock<broadcast::Sender<ButtonEvent>> = OnceLock::new();
static BUTTON_COUNTS_SENDER: OnceLock<watch::Sender<ButtonCounts>> = OnceLock::new();

///Broadcasts a button event to all listeners registered through boppo_core's HAL, on the wasm side.
pub(crate) fn broadcast_event(event: ButtonEvent) {
    let _ = BUTTON_SENDER.get().unwrap().send(event);
    BUTTON_COUNTS_SENDER.get().unwrap().send_modify(|counts| {
        counts.update_for_event(event.button(), event.is_pressed());
    });
}

pub(crate) fn init() {
    let (sender, _) = broadcast::channel::<ButtonEvent>(16);
    BUTTON_SENDER.set(sender.clone()).unwrap();
    boppo_core::hal::set_button_events(sender);
    let (button_counts_sender, button_counts_receiver) = watch::channel(ButtonCounts::default());
    BUTTON_COUNTS_SENDER.set(button_counts_sender).unwrap();
    boppo_core::hal::set_button_counts(button_counts_receiver);
}
