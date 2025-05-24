// use super::consts::*;
// use x86_64::structures::idt::{InterruptDescriptorTable,InterruptStackFrame};
// use crate::proc;
// use crate::{memory::gdt, proc::ProcessContext};
// // BootInfo,UefiRuntime,
// use core::sync::atomic::AtomicU64;
// use crate::guard_access_fn;
// use boot::{BootInfo,*};
// use uefi::runtime::{Time,*};
// use chrono::naive::{NaiveDate, NaiveDateTime};
// use super::consts::*;


// pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
//     unsafe{idt[Interrupts::IrqBase as u8 + Irq::Timer as u8]
//         .set_handler_fn(clock_handler)
//         .set_stack_index(gdt::CLOCK_IST_INDEX);}
// }

// pub extern "C" fn clock(mut context: proc::ProcessContext){
    
//     x86_64::instructions::interrupts::without_interrupts(|| {
//         proc::switch(&mut context);
//         super::ack();
//     });
// }

// as_handler!(clock);

// pub fn get_time() -> Time {
//     let ret = uefi::runtime::get_time().unwrap();
//     ret
// }

use super::consts::*;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::proc;
use crate::{memory::gdt, proc::ProcessContext};
use core::sync::atomic::AtomicU64;
use crate::guard_access_fn;
use boot::{BootInfo, *};
use uefi::runtime::{Time, *};
use chrono::{NaiveDateTime, Datelike, Timelike};
use chrono::naive::NaiveDate;
use chrono::Duration;

pub static SYSTEM_TIME: AtomicU64 = AtomicU64::new(0);

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    unsafe {
        idt[Interrupts::IrqBase as u8 + Irq::Timer as u8]
            .set_handler_fn(clock_handler)
            .set_stack_index(gdt::CLOCK_IST_INDEX);
    }
}

pub extern "C" fn clock(mut context: proc::ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // 更新系统时间计数器
        SYSTEM_TIME.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        proc::switch(&mut context);
        super::ack();
    });
}

as_handler!(clock);

/// 获取 UEFI 系统时间
pub fn get_uefi_time() -> Time {
    uefi::runtime::get_time().expect("Failed to get UEFI time")
}

/// 将 UEFI Time 转换为 NaiveDateTime
pub fn time_to_datetime(time: Time) -> NaiveDateTime {
    NaiveDate::from_ymd(
        time.year() as i32,
        time.month() as u32,
        time.day() as u32
    ).and_hms_nano(
        time.hour() as u32,
        time.minute() as u32,
        time.second() as u32,
        time.nanosecond() as u32
    )
}

/// 获取当前系统时间（从启动开始的毫秒数）
pub fn sys_time() -> Duration {
    let ticks = SYSTEM_TIME.load(core::sync::atomic::Ordering::Relaxed);
    // 假设时钟中断频率是 1000Hz，每个 tick 是 1ms
    Duration::milliseconds(ticks as i64)
}

/// 获取当前日期时间
pub fn current_datetime() -> NaiveDateTime {
    time_to_datetime(get_uefi_time())
}

/// Sleep 函数实现
pub fn sleep(millisecs: i64) {
    let start = sys_time();
    let dur = Duration::milliseconds(millisecs);
    
    while sys_time() - start < dur {
        // 让出 CPU 时间片
        // crate::proc::yield_now();
    }
}