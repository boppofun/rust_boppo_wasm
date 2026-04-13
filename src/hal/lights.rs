use boppo_core::color::RGB;
use boppo_core::hal::HalLights;

#[link(wasm_import_module = "host")]
unsafe extern "C" {
    fn boppo_wasm_set_color(idx: i32, r: i32, g: i32, b: i32);
    fn boppo_wasm_flush();
}

pub struct WasmHalLights;

impl HalLights for WasmHalLights {
    fn set_color(&mut self, idx: usize, color: RGB) {
        unsafe {
            boppo_wasm_set_color(idx as i32, color.r as i32, color.g as i32, color.b as i32);
        }
    }

    fn flush(&mut self) {
        unsafe {
            boppo_wasm_flush();
        }
    }
}

pub fn init_lights() {
    boppo_core::hal::set_lights(WasmHalLights);
}
