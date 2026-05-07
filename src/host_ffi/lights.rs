use std::os::raw::c_void;

use boppo_core::Lights;
use boppo_core::color::rgb::Rgb;

#[link(wasm_import_module = "host")]
unsafe extern "C" {
    /// Calls flush on the host
    fn boppo_wasm_set_and_flush_lights(framebuffer_colors: *const c_void);
}

fn set_and_flush_lights(colors: &[boppo_core::color::RGB; Lights::COUNT]) {
    unsafe {
        boppo_wasm_set_and_flush_lights(colors as *const [Rgb<u8>; Lights::COUNT] as *const c_void);
    }
}

pub fn init_lights() {
    boppo_core::hal::set_lights(set_and_flush_lights);
}
