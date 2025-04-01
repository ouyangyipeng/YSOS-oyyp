use log::{Level, LevelFilter, Metadata, Record};
use crate::drivers::serial::get_serial_for_sure;
use core::fmt::Write;

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Failed to set logger");

    info!("Logger Initialized");  // 移除了手动添加的[+]
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut serial = get_serial_for_sure();
            
            // 为不同级别生成不同前缀
            let (symbol, level_name) = match record.level() {
                Level::Error => ("\x1b[31m[X]\x1b[0m", "ERROR"),  // 保留ANSI代码供支持的环境
                Level::Warn => ("\x1b[33m[!]\x1b[0m", "WARN"),
                Level::Info => ("\x1b[34m[+]\x1b[0m", "INFO"),
                Level::Debug => ("\x1b[36m[#]\x1b[0m", "DEBUG"),
                Level::Trace => ("\x1b[32m[%]\x1b[0m", "TRACE"),
            };

            // 增强可读性的格式
            let _ = serial.write_fmt(format_args!(
                "{} \x1b[1m{}\x1b[0m - {}:{} - {}\n\r",
                symbol,
                level_name,
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            ));
        }
    }

    fn flush(&self) {}
}