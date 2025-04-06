use super::consts::*;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};


use crate::drivers::{input, serial::get_serial_for_sure};

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Serial0 as u8]
        .set_handler_fn(serial_handler);
    info!("Serial Interrupt Handler Registered.");
}

pub extern "x86-interrupt" fn serial_handler(_st: InterruptStackFrame) {
    receive();
    super::ack();
}

/// Receive character from uart 16550
/// Should be called on every interrupt
fn receive() {
    // FIXME: receive character from uart 16550, put it into INPUT_BUFFER
    // 获取串口实例
    let mut serial = get_serial_for_sure();
    
    // 循环读取所有可用的字符
    while let Some(c) = serial.receive() {
        // 将字符放入输入缓冲区
        input::push_key(c);
    }
}
