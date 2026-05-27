use boppo_core::log;

struct WasmLogger;

impl log::Log for WasmLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let level = match record.level() {
            log::Level::Error => "E",
            log::Level::Warn => "W",
            log::Level::Info => "I",
            log::Level::Debug => "D",
            log::Level::Trace => "T",
        };
        eprintln!("{} wasm: {}", level, record.args());
    }

    fn flush(&self) {}
}

static LOGGER: WasmLogger = WasmLogger;

pub fn init() {
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(log::LevelFilter::Info);
    }
}
