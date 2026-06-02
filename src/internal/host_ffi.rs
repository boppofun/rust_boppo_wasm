//! See and keep docs up to date at <https://developer.boppo.com/docs/wasm/>
//! (<https://github.com/boppofun/dev_docs/blob/main/docs/wasm.mdx>)

use std::ffi::c_void;
#[link(wasm_import_module = "host")]
unsafe extern "C" {
    pub(crate) fn boppo_poll(timeout_ms: i32) -> i64;

    pub(crate) fn boppo_set_and_flush_lights(framebuffer_colors: *const c_void);

    pub(crate) fn boppo_execute_command(cmd_ptr: *const u8, cmd_length: usize) -> i32;

    // ## Audio APIs

    pub(crate) fn boppo_play_sound_instruction(si_ptr: *const u8, si_length: usize) -> i32;

    pub(crate) fn boppo_set_controller_parameter(
        controller_id: u64,
        parameter: i32,
        value: f32,
    ) -> i32;

    pub(crate) fn boppo_stop_all_sounds();
}
