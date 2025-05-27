use log::{Level, LevelFilter, Metadata, Record};
use crate::drivers::serial::get_serial_for_sure;
use core::fmt::Write;
// use x86::io::{outb, inb};
use alloc::format;

// // 假设x86架构CMOS RTC
// unsafe fn read_rtc_seconds() -> u8 {
//     outb(0x70, 0x00); // 选择秒寄存器
//     inb(0x71)
// }

// // BCD转十进制
// fn bcd_to_dec(bcd: u8) -> u8 {
//     (bcd >> 4) * 10 + (bcd & 0x0F)
// }



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

// 实现一个计数，每一个log都增加一次
// static mut timestamp: usize = 0;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut serial = get_serial_for_sure();
            // 在Logger中调用
            // let seconds = unsafe { bcd_to_dec(read_rtc_seconds()) };
            // let timestamp = format!("\x1b[37m00:00:{:02}\x1b[0m", seconds).as_str();
            
            // 时间戳（假设你有一个适合 no_std 的时间库）
            let timestamp = 0; // 替换为实际的时间戳逻辑
            // unsafe{timestamp += 1;}
            // let num = unsafe{timestamp};
            
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
            // unsafe{
            let _ = write!(
                serial,
                "\x1b[37m[{}]\x1b[0m \x1b[35m[{}]\x1b[0m {} {} - {}:{}", 
                // num, module, symbol, level_name, file, line
                timestamp, module, symbol, level_name, file, line
            );
            // }
            let _ = serial.write_fmt(*record.args());
            let _ = serial.write_str("\n\r");
        }
    }

    fn flush(&self) {}
}