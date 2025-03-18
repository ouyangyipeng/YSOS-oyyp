use log::{Metadata, Record, LevelFilter, Level};
use crate::drivers::serial::get_serial_for_sure;
use core::fmt::Write;

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Failed to set logger.");

    // FIXME: Configure the logger

    info!("[+] Logger Initialized.");// 加上了[+]
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level() // 让log crate自己处理日志级别
    }

    fn log(&self, record: &Record) {
        // FIXME: Implement the logger with serial output
        if self.enabled(record.metadata()) {
            // 获取互斥锁保护的串口实例
            let mut serial = get_serial_for_sure();
            
            // 格式化输出到串口
            let _ = serial.write_fmt(format_args!(
                "[{}] {}:{} - {}\n\r",
                level_to_str(record.level()),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            ));
        }
    }

    fn flush(&self) {}
}

// Helper function to convert Level to string
fn level_to_str(level: Level) -> &'static str {
    match level {
        Level::Error => "ERROR",
        Level::Warn => "WARN",
        Level::Info => "INFO",
        Level::Debug => "DEBUG",
        Level::Trace => "TRACE",
    }
}
