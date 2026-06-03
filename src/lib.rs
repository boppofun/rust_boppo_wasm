//! Activity API for the [Boppo Tablet](https://developer.boppo.com).
//!
//! Activities are compiled to WASM and run on the tablet. The main entry point
//! for most activities is [`init_and_run_async`].
//!
//! ## Lights
//!
//! For setting the color of lights see the set_color and similar functions on
//! [`Button`], [`Buttons`], and [`Lights`].
//!
//! By default all changes are immediately flushed to the hardware. If you want
//! to make multiple changes in a row and performance matters you can use
//! [`Framebuffer`] or modify the auto flush behavior using [`MainFramebuffer`].
//!
//! Each Boppo button has 4 LED lights which are represented by [`LightDir`].
//!
//! For drawing and animating simple shapes treating the entire Boppo surface as
//! a display see [`lights_plane`].
//!
//! ## Button Events
//!
//! You can receive button change events as an async stream using
//! [`ButtonEvents`].
//!
//! You can also query the current state of the buttons using
//! [`Button::is_pressed`] and [`Buttons::currently_pressed`].
//!
//! You can use [`Button::wait_for_press`] and [`Button::wait_for_release`] to
//! wait for a button to be in a specific state.
//!
//! ## Audio
//!
//! Play audio using [`audio::play`] or the `audio::play_*` helper functions.
//!
//! ## Guidelines
//!
//! See Boppo's
//! [Activity Guidelines](https://developer.boppo.com/docs/activity-guidelines)
//! for guidelines on creating great activities.
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
