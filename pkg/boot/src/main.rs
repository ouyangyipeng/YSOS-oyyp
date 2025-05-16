#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
extern crate alloc;

use core::panic;

use alloc::boxed::Box;
use alloc::vec;
use config::Config;
use uefi::{entry, Status, mem::memory_map::MemoryMap};
use x86_64::registers::control::*;
use ysos_boot::*;
use xmas_elf::ElfFile;
use elf::*;
use x86_64::registers::control::*;
use x86_64::structures::paging::page::PageRange;
mod config;

const CONFIG_PATH: &str = "\\EFI\\BOOT\\boot.conf";

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().expect("Failed to initialize utilities");

    log::set_max_level(log::LevelFilter::Info);
    info!("Running UEFI bootloader...");

    // 1. Load config
    let config = { /* FIXME: Load config file as Config */ 
        let mut file = fs::open_file(CONFIG_PATH);
        let buf = fs::load_file(&mut file);
        config::Config::parse(buf)
    };

    trace!("Config: {:#x?}", config);

    // 2. Load ELF files
    let elf = { /* FIXME: Load kernel elf file */ 
        let mut file = fs::open_file(config.kernel_path);
        trace!("Loading kernel ELF file: {}", config.kernel_path);
        let buf = fs::load_file(&mut file);
        match ElfFile::new(buf) {
            Ok(elf) => elf,
            Err(e) => panic!("Failed to load ELF file: {:?}", e),
        }
    };
    trace!("ELF file loaded");

    unsafe {
        set_entry(elf.header.pt2.entry_point() as usize);
        trace!("Kernel entry point: {:#x}", elf.header.pt2.entry_point());
    }

    // 3. Load MemoryMap
    let mmap = uefi::boot::memory_map(MemoryType::LOADER_DATA).expect("Failed to get memory map");

    let max_phys_addr = mmap
        .entries()
        .map(|m| m.phys_start + m.page_count * 0x1000)
        .max()
        .unwrap()
        .max(0x1_0000_0000); // include IOAPIC MMIO area
    trace!("Max physical address: {:#x}", max_phys_addr);

    // 4. Map ELF segments, kernel stack and physical memory to virtual memory
    let mut page_table = current_page_table();

    // FIXME: root page table is read only, disable write protect (Cr0)
    unsafe{
        Cr0::update(|f| f.remove(Cr0Flags::WRITE_PROTECT));
    }
    trace!("Write protect disabled");

    // FIXME: map physical memory to specific virtual address offset
    let mut frame_allocator = UEFIFrameAllocator;
    map_physical_memory(config.physical_memory_offset, max_phys_addr, &mut page_table, &mut frame_allocator);
    trace!("Physical memory mapped to virtual memory");
    trace!("Physical memory offset: {:#x}", config.physical_memory_offset);

    // FIXME: load and map the kernel elf file
    match load_elf(&elf, config.physical_memory_offset, &mut page_table, &mut frame_allocator, false){
        Ok(_) => trace!("Kernel ELF loaded successfully"),
        Err(e) => panic!("Failed to load ELF file: {:?}", e),
    }

    // FIXME: map kernel stack
    let (stack_start, stack_size) = if config.kernel_stack_auto_grow > 0 {
        let stack_start = config.kernel_stack_address
            + (config.kernel_stack_size - config.kernel_stack_auto_grow) * 0x1000;
        (stack_start, config.kernel_stack_auto_grow)
    } else {
        (config.kernel_stack_address, config.kernel_stack_size)
    };

    trace!(
        "Kernel init stack: [0x{:x?} -> 0x{:x?}), size: {} pages",
        stack_start,
        stack_start + stack_size * 0x1000,
        stack_size
    );

    match map_range(
        stack_start,
        stack_size,
        &mut page_table,
        &mut frame_allocator,
        false,
        // false,
    )
    {
        Ok(range) => trace!("Kernel stack mapped: {:#x?}", range),
        Err(e) => panic!("Failed to map kernel stack: {:?}", e),
    }

    // FIXME: recover write protect (Cr0)
    unsafe {
        Cr0::update(|f| f.insert(Cr0Flags::WRITE_PROTECT));
    }
    trace!("Write protect enabled");

    free_elf(elf);
    trace!("ELF file freed");

    // Define Apps
    let apps = if config.load_apps {
        info!("Loading apps...");
        Some(load_apps())
    } else {
        info!("Skip loading apps");
        None
    };

    // 5. Pass system table to kernel
    let ptr = uefi::table::system_table_raw().expect("Failed to get system table");
    let system_table = ptr.cast::<core::ffi::c_void>();


    // 6. Exit boot and jump to ELF entry
    info!("Exiting boot services...");

    let mmap = unsafe { uefi::boot::exit_boot_services(MemoryType::LOADER_DATA) };
    // NOTE: alloc & log are no longer available

    // construct BootInfo
    let bootinfo = BootInfo {
        memory_map: mmap.entries().copied().collect(),
        physical_memory_offset: config.physical_memory_offset,
        system_table,
        loaded_apps: apps,
    };

    // align stack to 8 bytes
    let stacktop = config.kernel_stack_address + config.kernel_stack_size * 0x1000 - 8;

    info!("Bootloader started");

    jump_to_entry(&bootinfo, stacktop);
}
