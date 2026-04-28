use boppo_core::log;

struct WasmLogger;

impl log::Log for WasmLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        eprintln!("[{}] WASM Activity : {}", record.level(), record.args());
    }

    fn flush(&self) {}
}

static LOGGER: WasmLogger = WasmLogger;

pub fn init() {
    log::set_logger(&LOGGER).ok();
    log::set_max_level(log::LevelFilter::Debug);
}
