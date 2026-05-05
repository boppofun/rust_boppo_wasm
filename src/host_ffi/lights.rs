use std::cell::RefCell;
use std::os::raw::c_void;

use boppo_core::color::RGB;
use boppo_core::color::rgb::Rgb;
use boppo_core::hal::HalLights;
use boppo_core::{Framebuffer, Lights, NUM_LIGHTS};

#[link(wasm_import_module = "host")]
unsafe extern "C" {
    /// Calls flush on the host
    fn boppo_wasm_set_and_flush_lights(framebuffer_colors: *const c_void);
}

pub struct WasmHalLights {
    framebuffer: RefCell<Framebuffer>,
}

impl Default for WasmHalLights {
    fn default() -> Self {
        Self {
            framebuffer: RefCell::new(Framebuffer::new()),
        }
    }
}

impl HalLights for WasmHalLights {
    fn set_color(&self, idx: usize, color: RGB) {
        self.framebuffer
            .borrow_mut()
            .set_color(Lights::from_index(idx), color);
    }

    fn set_all_colors(&self, colors: &[boppo_core::color::RGB; NUM_LIGHTS]) {
        self.framebuffer.borrow_mut().set_all_colors(colors);
    }

    fn flush(&self) {
        // TODO lets pass in the lights here so we don't have to store them in framebuffer
        unsafe {
            boppo_wasm_set_and_flush_lights(
                &self.framebuffer.borrow().colors as *const [Rgb<u8>; NUM_LIGHTS] as *const c_void,
            );
        }
    }
}

pub fn init_lights() {
    boppo_core::hal::set_lights(WasmHalLights::default());
}
