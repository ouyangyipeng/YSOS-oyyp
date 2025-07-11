#![no_std]
#![no_main]
#![feature(alloc_error_handler)]  // 如果需要处理内存分配错误

extern crate alloc;
// use alloc::string::String;
// use alloc::format;
use drivers::ata::*;
use storage::mbr::*;
use storage::*;
#[macro_use]
extern crate log;

use ysos::*;

// use core::arch::asm;
use ysos_kernel as ysos;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);
    info!("Kernel initialized.");
    // let s = AtaDrive::open(0, 0).unwrap();
    open_drive();
    proc::list_app();
    ysos::wait(spawn_init());
    // spawn_init();
    ysos::shutdown();
}

pub fn spawn_init() -> proc::ProcessId {
    // NOTE: you may want to clear the screen before starting the shell
    // print!("\x1b[1;1H\x1b[2J");

    // proc::list_app();
    debug!("Spawn init process");
    // proc::spawn("hello").unwrap()
    proc::spawn("sh").unwrap()
}

pub fn open_drive(){
    let drive = AtaDrive::open(0, 0).unwrap();
    let mbrtab = MbrTable::parse(drive)
        .expect("Failed to parse MBR");
    let parts = mbrtab.partitions().expect("Failed to get partitions");
    let mut i = 0;
    for p in parts{
        // let inner = p.inner.clone();
        // let offset = p.offset;
        // let size = p.size;
        // let partition = Partition::new(inner, offset, size);
        // info!("Found partition#{} at offset {} with size {}", i, offset, size);
        info!("Found partition#{}: {:?}", i, p);
        i += 1;
    }
}