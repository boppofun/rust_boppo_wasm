//! Items internal to the WASM framework or used to initialize it.
//!
//! Activity developers should not need to use these items directly
//! unless they are manually initializing the WASM framework instead
//! of using the default `init_and_run_async`.
//!
//! Many items in internal are exported for the WASM host.

mod buttons;
pub(crate) mod host_ffi;
mod lights;
mod logger;
mod timer;
mod wasm_executor;

pub use boppo_core::internal::wasm::HostEvent;
pub use wasm_executor::block_on;

/// Initialize all WASM subsystems (logger, lights, buttons, executor, audio).
///
/// Called automatically by [`crate::init_and_run_async`]. Only call this directly
/// if you are manually driving the runtime instead of using that helper.
pub fn init() {
    logger::init();
    lights::init_lights();
    buttons::init();
    wasm_executor::init();
    crate::audio::init();
}

/// This is exported for the WASM host to read the SDK version at runtime.
#[unsafe(no_mangle)]
pub static SDK_VERSION: u64 = {
    let major = parse_u64(env!("CARGO_PKG_VERSION_MAJOR"));
    let minor = parse_u64(env!("CARGO_PKG_VERSION_MINOR"));
    let patch = parse_u64(env!("CARGO_PKG_VERSION_PATCH"));

    (major << 32) | (minor << 16) | patch
};

const fn parse_u64(s: &str) -> u64 {
    let bytes = s.as_bytes();
    let mut result = 0u64;
    let mut i = 0;
    while i < bytes.len() {
        result = result * 10 + (bytes[i] - b'0') as u64;
        i += 1;
    }
    result
}
