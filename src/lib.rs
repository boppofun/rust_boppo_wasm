#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

pub mod audio;
pub mod internal;

pub use boppo_core::*;

pub use boppo_core::internal::wasm::Error;

/// Initializes the Boppo WASM runtime and runs an async activity function.
///
/// If `activity_fn` returns, it is called again, as most Boppo activities
/// are expected to start fresh after completion. All audio is stopped and the
/// lights are turned off before starting again. The activity is passed in the
/// number of times it has been started (the first time is 1).
///
/// If you would like to return to the main menu, you can call std::process::exit(0).
///
/// ```no_run
/// use boppo_wasm::{Button, color};
///
/// pub fn main() {
///     boppo_wasm::init_and_run_async(activity)
/// }
///
/// pub async fn activity(num_starts: u32) {
///     Button::B0.set_color(color::BLUE);
///    // ...
/// }
/// ```
pub fn init_and_run_async(mut activity_fn: impl AsyncFnMut(u32)) {
    internal::init();
    internal::block_on(async {
        let mut num_starts = 1;
        loop {
            activity_fn(num_starts).await;
            audio::stop_all();
            Buttons::all().set_color(color::OFF);
            // Let's sleep a tiny bit in case there is an activity bug that
            // causes an activity to end immediately so there isn't a CPU
            // blocking loop.
            boppo_core::executor::sleep_ms(50).await;
            num_starts += 1;
        }
    })
}

/// Execute a command.
///
/// See the full list of supported commands at [developer.boppo.com/docs/commands](https://developer.boppo.com/docs/commands)
///
/// ## Examples
///
/// ```no_run
/// use boppo_wasm::execute_command;
///
/// // stop the activity and put the device to sleep
/// execute_command("sleep").unwrap();
///
/// // start a new activity
/// execute_command("start wasm ...").unwrap();
///
/// // Send a MIDI note "on" message over USB
/// execute_command("send_midi_usb_note_on 60").unwrap(); // play middle C
/// ```
pub fn execute_command(command: &str) -> Result<(), Error> {
    Error::result_from_i32(unsafe {
        internal::host_ffi::boppo_execute_command(command.as_ptr(), command.len())
    })?;
    Ok(())
}
