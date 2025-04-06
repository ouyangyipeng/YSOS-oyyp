use crate::memory::*;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

// use core::sync::atomic::{AtomicU64, Ordering};

use super::consts::*;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Timer as u8]
        .set_handler_fn(clock_handler)
        .set_stack_index(gdt::TIMER_IST_INDEX);
    info!("Clock Interrupt Handler Registered.");
}

pub extern "x86-interrupt" fn clock_handler(_sf: InterruptStackFrame) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        if inc_counter() % 0x10000 == 0 {
            trace!("Tick! @{}", read_counter());
        }
        super::ack();
    });
}

// static COUNTER: AtomicU64 = AtomicU64::new(0);

// #[inline]
// pub fn read_counter() -> u64 {
//     // FIXME: load counter value
//     COUNTER.load(Ordering::Relaxed)// 时间戳不需要严格顺序
// }

// #[inline]
// pub fn inc_counter() -> u64 {
//     // FIXME: read counter value and increase it
//     COUNTER.fetch_add(1, Ordering::SeqCst)// 必须使用SeqCst保证全局可见性
// }

// 尝试不使用atomic，因为debug出问题了

static mut COUNTER: u64 = 0;

#[inline]
pub fn read_counter() -> u64 {
    // FIXME: load counter value
    unsafe{
        COUNTER
    }
}

#[inline]
pub fn inc_counter() -> u64 {
    // FIXME: read counter value and increase it
    unsafe{
        COUNTER += 1;
        COUNTER
    }
}