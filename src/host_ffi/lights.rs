use std::os::raw::c_void;
use std::sync::Mutex;

use boppo_core::color::RGB;
use boppo_core::hal::HalLights;
use boppo_core::{Framebuffer, Lights, NUM_LIGHTS};

#[link(wasm_import_module = "host")]
unsafe extern "C" {
    /// Calls flush on the host
    fn boppo_wasm_set_and_flush_lights(framebuffer_colors: *const c_void);
}

pub struct WasmHalLights {
    framebuffer: Mutex<Framebuffer>,
}

impl Default for WasmHalLights {
    fn default() -> Self {
        Self {
            framebuffer: Mutex::new(Framebuffer::new()),
        }
    }
}

impl HalLights for WasmHalLights {
    fn set_color(&self, idx: usize, color: RGB) {
        let mut fb = self.framebuffer.lock().unwrap();
        fb.set_color(Lights::from_index(idx), color);
    }

    fn set_all_colors(&self, colors: &[boppo_core::color::RGB; NUM_LIGHTS]) {
        let mut fb = self.framebuffer.lock().unwrap();
        fb.set_all_colors(colors);
    }

    fn flush(&self) {
        unsafe {
            boppo_wasm_set_and_flush_lights(
                (&raw const self.framebuffer.lock().unwrap().colors).cast::<c_void>(),
            );
        }
    }
}

pub fn init_lights() {
    boppo_core::hal::set_lights(WasmHalLights::default());
}
