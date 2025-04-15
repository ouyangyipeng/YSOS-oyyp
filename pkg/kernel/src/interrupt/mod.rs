mod apic;
mod consts;
pub mod clock;
mod serial;
mod exceptions;

use apic::*;
use x86_64::structures::idt::InterruptDescriptorTable;
use crate::memory::physical_to_virtual;

use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {// 根据phil网站，这里必须是static，不然idt生命周期就不够长
        trace!("Creating IDT...");
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            trace!("Registering IDT...");
            exceptions::register_idt(&mut idt);
            clock::register_idt(&mut idt);
            serial::register_idt(&mut idt);
        }
        idt
    };
}



/// init interrupts system
pub fn init() {
    trace!("Initializing Interrupts...");
    IDT.load();


    trace!("IDT Loaded.");

    // FIXME: check and init APIC
    match apic::XApic::support() {
        true => {
            trace!("xAPIC supported.");
            let mut lapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
            trace!("Starting xAPIC...");
            lapic.cpu_init();
            trace!("xAPIC Initialized.");
        }
        false => {
            error!("xAPIC not supported.");
            panic!("xAPIC not supported.");
        }
        
    }

    // FIXME: enable serial irq with IO APIC (use enable_irq)
    enable_irq(consts::Irq::Serial0 as u8, 0); // enable IRQ4 for CPU0

    info!("Interrupts Initialized.");
}

#[inline(always)]
pub fn enable_irq(irq: u8, cpuid: u8) {
    let mut ioapic = unsafe { IoApic::new(physical_to_virtual(IOAPIC_ADDR)) };
    ioapic.enable(irq, cpuid);
}

#[inline(always)]
pub fn ack() {
    let mut lapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
    lapic.eoi();
}
