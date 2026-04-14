#[link(wasm_import_module = "host")]
unsafe extern "C" {
    /// Polling function for Button events with optionnal timeout.
    /// If timeout_ms <= 0, poll will happen indefinitely.
    /// This can be used to poll for button events or wait a certain time if not event was received
    /// in between.
    /// Returns a ButtonEvent u16 representation (>= 0) on any button event, -1 on timeout, -2 if channel diconnected early.
    pub fn boppo_wasm_poll(timeout_ms: i32) -> i32;
}
