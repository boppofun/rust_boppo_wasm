use std::os::raw::c_void;

use boppo_core::Lights;
use boppo_core::color::rgb::Rgb;

use crate::internal::host_ffi;

fn set_and_flush_lights(colors: &[boppo_core::color::RGB; Lights::COUNT]) {
    unsafe {
        host_ffi::boppo_set_and_flush_lights(
            colors as *const [Rgb<u8>; Lights::COUNT] as *const c_void,
        );
    }
}

pub fn init_lights() {
    boppo_core::internal::set_lights(set_and_flush_lights);
}
