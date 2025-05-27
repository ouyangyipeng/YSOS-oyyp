use x86_64::{
    structures::paging::{mapper::MapToError, page::*, Page},
    VirtAddr,
};
use crate::proc::*;

use super::{FrameAllocatorRef, MapperRef};
use core::ptr::copy_nonoverlapping;



use crate::proc::KERNEL_PID;
// 0xffff_ff00_0000_0000 is the kernel's address space
pub const STACK_MAX: u64 = 0x4000_0000_0000;
pub const STACK_MAX_PAGES: u64 = 0x100000;
pub const STACK_MAX_SIZE: u64 = STACK_MAX_PAGES * crate::memory::PAGE_SIZE;
pub const STACK_START_MASK: u64 = !(STACK_MAX_SIZE - 1);
// [bot..0x2000_0000_0000..top..0x3fff_ffff_ffff]
// init stack
pub const STACK_DEF_BOT: u64 = STACK_MAX - STACK_MAX_SIZE;
pub const STACK_DEF_PAGE: u64 = 1;
pub const STACK_DEF_SIZE: u64 = STACK_DEF_PAGE * crate::memory::PAGE_SIZE;

pub const STACK_INIT_BOT: u64 = STACK_MAX - STACK_DEF_SIZE;
pub const STACK_INIT_TOP: u64 = STACK_MAX - 8;

const STACK_INIT_TOP_PAGE: Page<Size4KiB> = Page::containing_address(VirtAddr::new(STACK_INIT_TOP));

// [bot..0xffffff0100000000..top..0xffffff01ffffffff]
// kernel stack
pub const KSTACK_MAX: u64 = 0xffff_ff02_0000_0000;
pub const KSTACK_DEF_BOT: u64 = KSTACK_MAX - STACK_MAX_SIZE;
pub const KSTACK_DEF_PAGE: u64 = 512;
pub const KSTACK_DEF_SIZE: u64 = KSTACK_DEF_PAGE * crate::memory::PAGE_SIZE;

pub const KSTACK_INIT_BOT: u64 = KSTACK_MAX - KSTACK_DEF_SIZE;
pub const KSTACK_INIT_TOP: u64 = KSTACK_MAX - 8;

const KSTACK_INIT_PAGE: Page<Size4KiB> = Page::containing_address(VirtAddr::new(KSTACK_INIT_BOT));
const KSTACK_INIT_TOP_PAGE: Page<Size4KiB> =
    Page::containing_address(VirtAddr::new(KSTACK_INIT_TOP));

pub struct Stack {
    pub range: PageRange<Size4KiB>,
    pub usage: u64,
}

impl Stack {
    pub fn new(top: Page, size: u64) -> Self {
        Self {
            range: Page::range(top - size + 1, top + 1),
            usage: size,
        }
    }

    pub const fn empty() -> Self {
        Self {
            range: Page::range(STACK_INIT_TOP_PAGE, STACK_INIT_TOP_PAGE),
            usage: 0,
        }
    }

    pub const fn kstack() -> Self {
        Self {
            range: Page::range(KSTACK_INIT_PAGE, KSTACK_INIT_TOP_PAGE),
            usage: KSTACK_DEF_PAGE,
        }
    }

    pub fn init(&mut self, mapper: MapperRef, alloc: FrameAllocatorRef) {
        debug_assert!(self.usage == 0, "Stack is not empty.");

        self.range = elf::map_range(STACK_INIT_BOT, STACK_DEF_PAGE, mapper, alloc, true).unwrap();
        self.usage = STACK_DEF_PAGE;
    }

    pub fn handle_page_fault(
        &mut self,
        addr: VirtAddr,             // 触发缺页异常的地址
        mapper: MapperRef,          // 映射器
        alloc: FrameAllocatorRef,   // 帧分配器
    ) -> bool {
        if !self.is_on_stack(addr) {
            return false;
        }

        if let Err(m) = self.grow_stack(addr, mapper, alloc) {
            error!("Grow stack failed: {:?}", m);
            return false;
        }

        true
    }

    fn is_on_stack(&self, addr: VirtAddr) -> bool {
        let addr = addr.as_u64();
        let cur_stack_bot = self.range.start.start_address().as_u64();
        trace!("Current stack bot: {:#x}", cur_stack_bot);
        trace!("Address to access: {:#x}", addr);
        addr & STACK_START_MASK == cur_stack_bot & STACK_START_MASK
    }

    fn grow_stack(
        &mut self,
        addr: VirtAddr,
        mapper: MapperRef,
        alloc: FrameAllocatorRef,
    ) -> Result<(), MapToError<Size4KiB>> {
        debug_assert!(self.is_on_stack(addr), "Address is not on stack.");

        // FIXME: grow stack for page fault
        let new_start = Page::<Size4KiB>::containing_address(addr);
        let pre_start = self.range.start;
        let pre_end = self.range.end;
        let new_count = pre_start - new_start;// 不太确定，要不要+1？
        let new_range = Page::range(new_start, pre_end);
        let new_usage = pre_end - pre_start;
        let is_user_access = processor::get_pid() != KERNEL_PID;
        
        match elf::map_range(
            new_start.start_address().as_u64(),
            new_count,
            mapper,
            alloc,
            is_user_access,
            // false,
        ) {
            Ok(range) => {
                // info!("Stack range: {:#?}", range);
                self.range = new_range;
                self.usage += new_usage;
            }
            Err(e) => {
                error!("Failed to map stack: {:#?}", e);
                return Err(e);
            }
        }

        self.usage = new_usage;
        self.range = new_range;

        trace!(
            "Stack range: {:#x} -> {:#x}",
            self.range.start.start_address().as_u64(),
            self.range.end.start_address().as_u64()
        );
        trace!(
            "Stack usage: {:#x} -> {:#x}",
            self.range.start.start_address().as_u64(),
            self.usage
        );

        Ok(())
    }

    pub fn memory_usage(&self) -> u64 {
        self.usage * crate::memory::PAGE_SIZE
    }

    pub fn fork(
        &self,
        mapper: MapperRef,
        alloc: FrameAllocatorRef,
        stack_offset_count: u64,
    ) -> Self {
        // FIXME: alloc & map new stack for child (see instructions)
        let mut new_start = self.range.start;
        let mut child_stack_top = 
            (new_start - stack_offset_count)
            .start_address()
            .as_u64();
        let child_stack_page_count = self.usage;

        // FIXME: copy the *entire stack* from parent to child
        while elf::map_range(
            child_stack_top,
            child_stack_page_count,
            mapper,
            alloc,
            true,
        ).is_err()
        {
            trace!("Map thread stack to {:#x} failed.", child_stack_top);
            child_stack_top -= STACK_MAX_SIZE;
        }

        let parent_addr = self.range.start.start_address().as_u64();
        let child_addr = child_stack_top;
        let size = child_stack_page_count;

        self.clone_range(
            parent_addr,
            child_addr,
            size,
        );

        let child_start_page =
            Page::<Size4KiB>::containing_address(VirtAddr::new(child_stack_top));
        let child_end_page = child_start_page + child_stack_page_count - 1;
        let child_range = Page::range(child_start_page, child_end_page + 1);
        trace!(
            "Child stack range: {:#x} -> {:#x}",
            child_range.start.start_address().as_u64(),
            child_range.end.start_address().as_u64()
        );
        trace!(
            "Child stack usage: {:#x} -> {:#x}",
            child_range.start.start_address().as_u64(),
            child_stack_page_count
        );

        // FIXME: return the new stack
        Self {
            range: child_range,
            usage: child_stack_page_count,
        }
    }
    /// Clone a range of memory
    ///
    /// - `src_addr`: the address of the source memory
    /// - `dest_addr`: the address of the target memory
    /// - `size`: the count of pages to be cloned
    fn clone_range(&self, cur_addr: u64, dest_addr: u64, size: u64) {
        trace!("Clone range: {:#x} -> {:#x}", cur_addr, dest_addr);
        unsafe {
            copy_nonoverlapping::<u64>(
                cur_addr as *mut u64,
                dest_addr as *mut u64,
                (size * Size4KiB::SIZE / 8) as usize,
            );
        }
    }
}

impl core::fmt::Debug for Stack {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Stack")
            .field(
                "top",
                &format_args!("{:#x}", self.range.end.start_address().as_u64()),
            )
            .field(
                "bot",
                &format_args!("{:#x}", self.range.start.start_address().as_u64()),
            )
            .finish()
    }
}
