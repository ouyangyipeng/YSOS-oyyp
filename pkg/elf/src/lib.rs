#![no_std]
// #![feature(portable_simd)]

#[macro_use]
extern crate log;
extern crate alloc;

use core::ptr::{copy_nonoverlapping, write_bytes};

use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::{mapper::*, *};
use x86_64::{align_up, PhysAddr, VirtAddr};
use xmas_elf::{program, ElfFile};
use x86_64::structures::paging::page::PageRangeInclusive;
use alloc::vec::Vec;

/// Map physical memory
///
/// map [0, max_addr) to virtual space [offset, offset + max_addr)
pub fn map_physical_memory(
    offset: u64,
    max_addr: u64,
    page_table: &mut impl Mapper<Size2MiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    trace!("Mapping physical memory...");
    let start_frame = PhysFrame::containing_address(PhysAddr::new(0));
    let end_frame = PhysFrame::containing_address(PhysAddr::new(max_addr));

    for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
        let page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64() + offset));
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            page_table
                .map_to(page, frame, flags, frame_allocator)
                .expect("Failed to map physical memory")
                .flush();
        }
    }
}

/// Map a range of memory
///
/// allocate frames and map to specified address (R/W)
pub fn map_range(
    addr: u64,
    count: u64,
    page_table: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    user_access: bool,
    // no_execute: bool,
) -> Result<PageRange, MapToError<Size4KiB>> {
    let range_start = Page::containing_address(VirtAddr::new(addr));
    let range_end = range_start + count;

    trace!(
        "Page Range: {:?}({})",
        Page::range(range_start, range_end),
        count
    );

    // default flags for stack
    let mut flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    if user_access {
        flags |= PageTableFlags::USER_ACCESSIBLE;
    }
    // if no_execute {
    //     flags.insert(PageTableFlags::NO_EXECUTE);
    // } else {
    //     flags.remove(PageTableFlags::NO_EXECUTE);
    // }
    trace!("Page table flag: {:?}", flags);

    for page in Page::range(range_start, range_end) {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        unsafe {
            page_table
                .map_to(page, frame, flags, frame_allocator)?
                .flush();
        }
    }

    trace!(
        "Map hint: {:#x} -> {:#x}",
        addr,
        page_table
            .translate_page(range_start)
            .unwrap()
            .start_address()
    );

    Ok(Page::range(range_start, range_end))
}


/// Unmap a range of memory
/// 
/// unmap specified address (R/W)
pub unsafe fn unmap_range(
    page_table: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameDeallocator<Size4KiB>,
    page_range: PageRange,
    dealloc: bool
) -> Result<(),UnmapError> {
    for page in page_range {
        let (frame, flush) = page_table.unmap(page)?;
        if dealloc {
            unsafe{
                frame_allocator.deallocate_frame(frame);
            }
        }
        flush.flush();
    }
    Ok(())
}


/// Load & Map ELF file
///
/// load segments in ELF file to new frames and set page table
pub fn load_elf(
    elf: &ElfFile,
    physical_offset: u64,
    page_table: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    user_access: bool,
// ) -> Result<(), MapToError<Size4KiB>> {
) -> Result<Vec<PageRangeInclusive>, MapToError<Size4KiB>> {
    let file_buf = elf.input.as_ptr();

    trace!("Loading ELF file... @ {:#x}", file_buf as u64);

    // for segment in elf.program_iter() {
    //     trace!("Loading segment: {:#x?}", segment);
    //     if segment.get_type().unwrap() != program::Type::Load {
    //         trace!("Skip segment: {:#x?}", segment);
    //         continue;
    //     }

    //     load_segment(
    //         file_buf,
    //         physical_offset,
    //         &segment,
    //         page_table,
    //         frame_allocator,
    //         user_access,
    //     )?
    // }
    // Ok(())
    elf.program_iter()
        .filter(|segment| segment.get_type().unwrap() == program::Type::Load)
        .map(|segment| {
            load_segment(
                elf,
                physical_offset,
                &segment,
                page_table,
                frame_allocator,
                user_access,
            )
        })
        .collect()
}

/// Load & Map ELF segment
///
/// load segment to new frame and set page table
fn load_segment(
    file_buf: &ElfFile,
    physical_offset: u64,
    segment: &program::ProgramHeader,
    page_table: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    user_access: bool,
// ) -> Result<(), MapToError<Size4KiB>> {
) -> Result<PageRangeInclusive, MapToError<Size4KiB>> {
    trace!("Loading & mapping segment: {:#x?}", segment);

    let mem_size = segment.mem_size();
    let file_size = segment.file_size();
    let file_offset = segment.offset() & !0xfff;
    let virt_start_addr = VirtAddr::new(segment.virtual_addr());

    let mut page_table_flags = PageTableFlags::PRESENT;

    // FIXME: handle page table flags with segment flags
    // unimplemented!("Handle page table flags with segment flags!");

    // let flags = segment.flags();

    // // 写
    // if flags.is_write() {
    //     page_table_flags |= PageTableFlags::WRITABLE;
    // }
    // info!("Segment flags: {:?}", flags);

    // // 执行
    // if !flags.is_execute() {
    //     page_table_flags |= PageTableFlags::NO_EXECUTE;
    // }

    if segment.flags().is_execute() {
        page_table_flags.remove(PageTableFlags::NO_EXECUTE);
    } else {
        page_table_flags.insert(PageTableFlags::NO_EXECUTE);
    };
    if segment.flags().is_write() {
        page_table_flags.insert(PageTableFlags::WRITABLE);
    } else {
        page_table_flags.remove(PageTableFlags::WRITABLE);
    };
    if segment.flags().is_read() {
        page_table_flags.insert(PageTableFlags::USER_ACCESSIBLE);
    } else {
        page_table_flags.remove(PageTableFlags::USER_ACCESSIBLE);
    };
    if user_access {
        page_table_flags |= PageTableFlags::USER_ACCESSIBLE;
    }

    // if user_access {
    //     page_table_flags |= PageTableFlags::USER_ACCESSIBLE;
    // }

    trace!("Segment page table flag: {:?}", page_table_flags);

    let start_page = Page::containing_address(virt_start_addr);
    let end_page = Page::containing_address(virt_start_addr + file_size - 1u64);
    let pages = Page::range_inclusive(start_page, end_page);

    // let data = unsafe { file_buf.add(file_offset as usize) };
    let data = unsafe { file_buf.input.as_ptr().add(file_offset as usize) };

    for (idx, page) in pages.enumerate() {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let offset = idx as u64 * page.size();
        let count = if file_size - offset < page.size() {
            file_size - offset
        } else {
            page.size()
        };

        unsafe {
            copy_nonoverlapping(
                data.add(idx * page.size() as usize),
                (frame.start_address().as_u64() + physical_offset) as *mut u8,
                count as usize,
            );

            page_table
                .map_to(page, frame, page_table_flags, frame_allocator)?
                .flush();

            if count < page.size() {
                // zero the rest of the page
                trace!(
                    "Zeroing rest of the page: {:#x}",
                    page.start_address().as_u64()
                );
                write_bytes(
                    (frame.start_address().as_u64() + physical_offset + count) as *mut u8,
                    0,
                    (page.size() - count) as usize,
                );
            }
        }
    }

    if mem_size > file_size {
        // .bss section (or similar), which needs to be zeroed
        let zero_start = virt_start_addr + file_size;
        let zero_end = virt_start_addr + mem_size;

        // Map additional frames.
        let start_address = VirtAddr::new(align_up(zero_start.as_u64(), Size4KiB::SIZE));
        let start_page: Page = Page::containing_address(start_address);
        let end_page = Page::containing_address(zero_end);

        for page in Page::range_inclusive(start_page, end_page) {
            let frame = frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;

            unsafe {
                page_table
                    .map_to(page, frame, page_table_flags, frame_allocator)?
                    .flush();

                // zero bss section
                write_bytes(
                    (frame.start_address().as_u64() + physical_offset) as *mut u8,
                    0,
                    page.size() as usize,
                );
            }
        }
    }

    // Ok(())
    let end_page = Page::containing_address(virt_start_addr + mem_size - 1u64);
    Ok(Page::range_inclusive(start_page, end_page))
}
