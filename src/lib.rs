#[cfg(feature = "wasm_client")]
pub mod audio;
pub mod internal;

mod error;

#[cfg(feature = "wasm_client")]
pub use boppo_core::*;

pub use error::Error;

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
#[cfg(feature = "wasm_client")]
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
