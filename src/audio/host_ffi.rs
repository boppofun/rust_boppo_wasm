#[link(wasm_import_module = "host")]
unsafe extern "C" {
    pub(crate) fn boppo_play_sound_instruction(si_ptr: *const u8, si_length: usize) -> i32;

    pub(crate) fn boppo_set_controller_parameter(
        controller_id: i32,
        parameter: i32,
        value: f32,
    ) -> i32;

    pub(crate) fn boppo_stop_all_sounds();
}
