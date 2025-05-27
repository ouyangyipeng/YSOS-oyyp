use alloc::format;
use x86_64::{
    structures::paging::{page::*, *},
    VirtAddr,
};
use xmas_elf::ElfFile;

use crate::{humanized_size, memory::*, proc::vm::stack::Stack};

pub mod stack;
use crate::proc::KERNEL_PID;
use self::stack::*;

use super::{PageTableContext, ProcessId};

type MapperRef<'a> = &'a mut OffsetPageTable<'static>;
type FrameAllocatorRef<'a> = &'a mut BootInfoFrameAllocator;

pub struct ProcessVm {
    // page table is shared by parent and child
    pub(super) page_table: PageTableContext,

    // stack is pre-process allocated
    pub(super) stack: Stack,
}

impl ProcessVm {
    pub fn new(page_table: PageTableContext) -> Self {
        Self {
            page_table,
            stack: Stack::empty(),
        }
    }

    pub fn init_kernel_vm(mut self) -> Self {
        // TODO: record kernel code usage
        self.stack = Stack::kstack();
        self
    }

    // pub fn init_proc_stack(&mut self, pid: ProcessId) -> VirtAddr {
    //     // FIXME: calculate the stack for pid
    //     info!("Init process stack for pid: {}", pid);
    //     let stack_top_addr = STACK_INIT_TOP-((pid.0 as u64 -1) * 0x1_0000_0000);
    //     let stack_bot_addr = STACK_INIT_BOT-((pid.0 as u64 -1) * 0x1_0000_0000);
    //     info!("Stack top addr: {:#x}", stack_top_addr);
    //     let page_table = &mut self.page_table.mapper();
    //     // info!("Stack top addr: {:#x}", stack_top_addr);
    //     let alloc = &mut *get_frame_alloc_for_sure();
    //     // info!("Mapping stack for pid: {}", pid);
    //     let is_user_access = pid != KERNEL_PID;
    //     info!("stack_bot_addr: {:#x}", stack_bot_addr);
    //     match elf::map_range(
    //         stack_bot_addr,
    //         1,
    //         page_table,
    //         alloc,
    //         is_user_access,
    //         // false,
    //     ){
    //         Ok(range) => {
    //             trace!("Stack range: {:#?}", range);
    //         }
    //         Err(e) => {
    //             panic!("Failed to map stack: {:#?}", e);
    //         }
    //     }
    //     self.stack = Stack::new(
    //         Page::containing_address(VirtAddr::new(stack_top_addr)),
    //         STACK_DEF_PAGE,
    //     );
    //     info!("Stack: {:#?}", self.stack);
    //     VirtAddr::new(stack_top_addr)
    // }

    pub fn handle_page_fault(&mut self, addr: VirtAddr) -> bool {
        let mapper = &mut self.page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        self.stack.handle_page_fault(addr, mapper, alloc)
    }

    pub(super) fn memory_usage(&self) -> u64 {
        self.stack.memory_usage()
    }

    pub fn load_elf(
        &mut self,
        elf: &ElfFile
    ) {
        let mapper = &mut self.page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        // init stack
        self.stack.init(mapper, alloc);
        // match elf::load_elf(
        //     elf,
        //     *PHYSICAL_OFFSET.get().unwrap(),
        //     mapper,
        //     alloc,
        //     true
        // ) {
        //     Ok(_) => {
        //         info!("Process Loaded: {:#?}", elf);
        //     }
        //     Err(e) => {
        //         error!("Failed to load ELF file: {:?}", e);
        //         panic!("Failed to load ELF file: {:?}", e);
        //     }
        // }
        elf::load_elf(elf, PHYSICAL_OFFSET.get().cloned().unwrap(), mapper, alloc,true).unwrap();
    }

    pub fn fork(&self, stack_offset_count: u64) -> Self {
        // clone the page table context (see instructions)
        let owned_page_table = self.page_table.fork();

        let mapper = &mut owned_page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        Self {
            page_table: owned_page_table,
            stack: self.stack.fork(mapper, alloc, stack_offset_count),
        }
    }
}

impl core::fmt::Debug for ProcessVm {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (size, unit) = humanized_size(self.memory_usage());

        f.debug_struct("ProcessVm")
            .field("stack", &self.stack)
            .field("memory_usage", &format!("{} {}", size, unit))
            .field("page_table", &self.page_table)
            .finish()
    }
}
