#[cfg(feature = "wasm_client")]
pub mod audio;
pub mod internal;

#[cfg(feature = "wasm_client")]
mod error;

#[cfg(feature = "wasm_client")]
pub use boppo_core::*;
#[cfg(feature = "wasm_client")]
pub use error::Error;

/// Initializes the Boppo WASM runtime and runs an async activity function.
///
/// If the async function returns, it is called again, as most Boppo activities
/// are expected to restart.
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
/// pub async fn activity() {
///     Button::B0.set_color(color::BLUE);
///    // ...
/// }
/// ```
#[cfg(feature = "wasm_client")]
pub fn init_and_run_async(mut activity_fn: impl AsyncFnMut()) {
    internal::init();
    internal::block_on(async {
        loop {
            activity_fn().await
        }
    })
}
