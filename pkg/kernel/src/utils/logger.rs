use log::{Level, LevelFilter, Metadata, Record};
use crate::drivers::serial::get_serial_for_sure;
use core::fmt::Write;

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Failed to set logger");

    // 来测试一下
    log::error!("This is an error message");
    log::warn!("This is a warning message");
    log::info!("This is an info message");
    log::debug!("This is a debug message");
    log::trace!("This is a trace message");

    info!("Logger Initialized");
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut serial = get_serial_for_sure();
            
            // 时间戳（假设你有一个适合 no_std 的时间库）
            let timestamp = "00:00:00"; // 替换为实际的时间戳逻辑
            
            // 模块名
            let module = record.module_path().unwrap_or("unknown");
            
            // 日志级别和符号
            let (symbol, level_name) = match record.level() {
                Level::Error => (
                    "\x1b[5m\x1b[41m\x1b[37m[X]\x1b[0m",  // 闪烁 + 红色背景 + 白色符号
                    "\x1b[5m\x1b[31mERROR\x1b[0m"         // 闪烁 + 红色文字
                ),
                Level::Warn => (
                    "\x1b[33m[!]\x1b[0m",                 // 黄色符号
                    "\x1b[33mWARN\x1b[0m"                 // 黄色文字
                ),
                Level::Info => (
                    "\x1b[34m[+]\x1b[0m",                 // 蓝色符号
                    "\x1b[34mINFO\x1b[0m"                 // 蓝色文字
                ),
                Level::Debug => (
                    "\x1b[36m[#]\x1b[0m",                 // 青色符号
                    "\x1b[36mDEBUG\x1b[0m"                // 青色文字
                ),
                Level::Trace => (
                    "\x1b[32m[%]\x1b[0m",                 // 绿色符号
                    "\x1b[32mTRACE\x1b[0m"                // 绿色文字
                ),
            };

            // 文件名和行号
            let file = record.file().unwrap_or("unknown");
            let line = record.line().unwrap_or(0);

            // 组合日志格式
            let _ = write!(
                serial,
                "\x1b[37m[{}]\x1b[0m \x1b[35m[{}]\x1b[0m {} {} - {}:{}", 
                timestamp, module, symbol, level_name, file, line
            );
            let _ = serial.write_fmt(*record.args());
            let _ = serial.write_str("\n\r");
        }
    }

    fn flush(&self) {}
}