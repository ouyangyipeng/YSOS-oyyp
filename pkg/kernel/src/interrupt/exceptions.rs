use crate::memory::*;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::VirtAddr;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    // 0
    idt.divide_error.set_handler_fn(divide_error_handler);
    // 8
    idt.double_fault
        .set_handler_fn(double_fault_handler)
        .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    // 14
    idt.page_fault
        .set_handler_fn(page_fault_handler)
        .set_stack_index(gdt::PAGE_FAULT_IST_INDEX);

    // TODO: you should handle more exceptions here
    // especially general protection fault (GPF)
    // see: https://wiki.osdev.org/Exceptions

    // 需要独立栈的异常注册
    // 12
    idt.stack_segment_fault
        .set_handler_fn(stack_segment_fault_handler)
        .set_stack_index(gdt::STACK_SEGMENT_IST_INDEX);
    
    // 13
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault_handler)
        .set_stack_index(gdt::GPF_IST_INDEX);
    
    // 18
    idt.machine_check
        .set_handler_fn(machine_check_handler)
        .set_stack_index(gdt::MACHINE_CHECK_IST_INDEX);
    
    // 2
    // idt.non_maskable_interrupt
    //     .set_handler_fn(nmi_handler)
    //     .set_stack_index(gdt::NMI_IST_INDEX);

    // 不需要独立栈的常见异常
    // 3
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    // 6
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    // 11
    idt.segment_not_present.set_handler_fn(segment_not_present_handler);
    // 10
    idt.invalid_tss.set_handler_fn(invalid_tss_handler);
    // 17
    idt.alignment_check.set_handler_fn(alignment_check_handler);
    // 还剩下1、4、5、7、9、15、16、19、20、30
    // 7
    // idt.device_not_available.set_handler_fn(device_not_available_handler);
    // 1
    // idt.debug.set_handler_fn(debug_handler);
    // 16
    // idt.x87_floating_point.set_handler_fn(x87_floating_point_handler);
    // 19
    // idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
    // 4
    // idt.overflow.set_handler_fn(overflow_handler);
    // 5
    // idt.bound_range_exceeded.set_handler_fn(bound_range_handler);
    // 其他的几个还没怎么看懂
}

// 0
pub extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DIVIDE ERROR\n\n{:#?}", stack_frame);
}

// 8
pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT, ERROR_CODE: 0x{:016x}\n\n{:#?}",
        error_code, stack_frame
    );
}

// 14
pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    err_code: PageFaultErrorCode,
) {
    panic!(
        "EXCEPTION: PAGE FAULT, ERROR_CODE: {:?}\n\nTrying to access: {:#x}\n{:#?}",
        err_code,
        Cr2::read().unwrap_or(VirtAddr::new_truncate(0xdeadbeef)),
        stack_frame
    );
}


/* ----需要独立栈的处理函数---- */
// Stack Segment Fault (12)
pub extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: STACK SEGMENT FAULT, ERROR_CODE: 0x{:x}\n{:#?}",
        error_code, stack_frame
    );
}

// General Protection Fault (13)
pub extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT, ERROR_CODE: 0x{:x}\n{:#?}",
        error_code, stack_frame
    );
}

// Machine Check (18)
pub extern "x86-interrupt" fn machine_check_handler(
    stack_frame: InterruptStackFrame,
) -> ! {
    panic!(
        "CRITICAL EXCEPTION: MACHINE CHECK\n{:#?}",
        stack_frame
    );
}

// nmi似乎不该在这里处理？
/*
// Non-Maskable Interrupt (2)
pub extern "x86-interrupt" fn nmi_handler(
    stack_frame: InterruptStackFrame,
) {
    // NMI 还需要特殊处理
    log::error!("NMI occurred, possible hardware failure!");
    // 加硬件诊断代码……
    panic!("UNRECOVERABLE NMI\n{:#?}", stack_frame);
}
*/



/* ----常规处理函数---- */

// Debug (1) - 调试陷阱
// pub extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
//     println!("[DEBUG TRAP] at {:#x}", stack_frame.instruction_pointer);
//     这里还不太完整捏
// }

// Breakpoint (3)
pub extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("Debug breakpoint at {:#x}", stack_frame.instruction_pointer);
    // 调试用，可以继续执行，不用panic的
}

// Overflow (4)
// pub extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) -> ! {
//     panic!("Arithmetic Overflow at {:#x}", stack_frame.instruction_pointer);
// }

// Bound Range (5)
// pub extern "x86-interrupt" fn bound_range_handler(stack_frame: InterruptStackFrame) -> ! {
//     panic!("Bound Range Exceeded at {:#x}", stack_frame.instruction_pointer);
// }

// Invalid Opcode (6)
pub extern "x86-interrupt" fn invalid_opcode_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!(
        "EXCEPTION: INVALID OPCODE\nInstruction Pointer: {:#x}\n{:#?}",
        stack_frame.instruction_pointer, stack_frame
    );
}

// Device Not Available (7)
// pub extern "x86-interrupt" fn device_not_available_handler(
//     stack_frame: InterruptStackFrame,
// ) {
//     unsafe { 
//         crate::arch::x86_64::fpu::init_fpu(); 
//     }
// }


// Segment Not Present (11)
pub extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: SEGMENT NOT PRESENT, ERROR_CODE: 0x{:x}\n{:#?}",
        error_code, stack_frame
    );
}

// Invalid TSS (10)
pub extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: INVALID TSS, ERROR_CODE: 0x{:x}\n{:#?}",
        error_code, stack_frame
    );
}

// x87 Floating Point (16)
// pub extern "x86-interrupt" fn x87_floating_point_handler(
//     stack_frame: InterruptStackFrame,
// ) {
//     let status = unsafe { x86_64::registers::control::Cr0::read() };
//     log::warn!("FPU Exception (CR0: {:x})", status);
//     // 清除异常标志后继续执行
// }

// Alignment Check (17)
pub extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: ALIGNMENT CHECK FAILED, ERROR_CODE: 0x{:x}\n{:#?}",
        error_code, stack_frame
    );
}

// // SIMD Floating Point (19)
// pub extern "x86-interrupt" fn simd_floating_point_handler(
//     stack_frame: InterruptStackFrame,
// ) {
//     let mxcsr = unsafe { 
//         let mut val: u32;
//         core::arch::asm!("stmxcsr [{}]", in(reg) &mut val);
//         val
//     };
//     log::warn!("SIMD Exception (MXCSR: {:x})", mxcsr);
// }