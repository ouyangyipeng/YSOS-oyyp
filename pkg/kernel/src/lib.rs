#![no_std]
#![allow(dead_code)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(type_alias_impl_trait)]
#![feature(map_try_insert)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::result_unit_err)]

extern crate alloc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
extern crate libm;

#[macro_use]
pub mod utils;
pub use utils::*;

#[macro_use]
pub mod drivers;
pub use drivers::{*, ata::AtaDrive};

pub mod memory;
pub mod proc;
pub mod interrupt;

pub use alloc::format;

use boot::BootInfo;
use uefi::{Status, runtime::ResetType};


pub fn init(boot_info: &'static BootInfo) {
    unsafe {
        // set uefi system table
        uefi::table::set_system_table(boot_info.system_table.cast().as_ptr());
    }

    serial::init(); // init serial output
    logger::init(); // init logger system
    memory::address::init(boot_info);
    memory::allocator::init(); // init kernel heap allocator
    memory::gdt::init(); // init gdt
    trace!("Debug: Kernel Heap Initialized.");
    memory::init(boot_info); // init memory manager
    interrupt::init(); // init interrupts

    proc::init(boot_info); // init process manager
    trace!("Debug: Process Manager Initialized.");

    x86_64::instructions::interrupts::enable();
    info!("Interrupts Enabled.");

    info!("YatSenOS initialized.");
    
    drivers::filesystem::init();
    info!("Filesystem initialized.");
    AtaDrive::open(0, 0);
    
    info!("Test stack grow.");

    grow_stack();

    info!("Stack grow test done.");
}

pub fn shutdown() -> ! {
    info!("YatSenOS shutting down.");
    uefi::runtime::reset(ResetType::SHUTDOWN, Status::SUCCESS, None);
}

pub fn humanized_size(size: u64) -> (f64, &'static str) {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    if size == 0 {
        return (0.0, UNITS[0]);
    }

    let index = libm::floor(libm::log(size as f64) / libm::log(1024.0)) as usize;
    let index = index.min(UNITS.len() - 1);

    let converted_size = size as f64 / libm::pow(1024.0, index as f64);

    (converted_size, UNITS[index])
}

pub fn wait(init: proc::ProcessId) {
    loop {
        if proc::still_alive(init) {
            // Why? Check reflection question 5
            x86_64::instructions::hlt();
        } else {
            break;
        }
    }
}

#[inline(never)]
#[unsafe(no_mangle)]
pub fn grow_stack() {
    const STACK_SIZE: usize = 1024 * 4;
    const STEP: usize = 64;

    let mut array = [0u64; STACK_SIZE];
    info!("Stack: {:?}", array.as_ptr());

    // test write
    for i in (0..STACK_SIZE).step_by(STEP) {
        array[i] = i as u64;
    }

    // test read
    for i in (0..STACK_SIZE).step_by(STEP) {
        assert_eq!(array[i], i as u64);
    }
}