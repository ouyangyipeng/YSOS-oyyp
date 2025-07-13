use alloc::{format, vec::Vec};
use x86_64::{
    structures::paging::{
        mapper::{CleanUp, UnmapError},
        page::*,
        *,
    },
    VirtAddr,
};
use xmas_elf::ElfFile;
use crate::{humanized_size, memory::*};

pub mod heap;
pub mod stack;

use self::{heap::Heap, stack::Stack};

use super::PageTableContext;

use boot::KernelPages;

// See the documentation for the `KernelPages` type
// Ignore when you not reach this part
//
// use boot::KernelPages;

type MapperRef<'a> = &'a mut OffsetPageTable<'static>;
type FrameAllocatorRef<'a> = &'a mut BootInfoFrameAllocator;

pub struct ProcessVm {
    // page table is shared by parent and child
    pub(super) page_table: PageTableContext,

    // stack is pre-process allocated
    pub(super) stack: Stack,

    // heap is allocated by brk syscall
    pub(super) heap: Heap,

    // code is hold by the first process
    // these fields will be empty for other processes
    pub(super) code: Vec<PageRangeInclusive>,
    pub(super) code_usage: u64,
}

impl ProcessVm {
    pub fn new(page_table: PageTableContext) -> Self {
        Self {
            page_table,
            stack: Stack::empty(),
            heap: Heap::empty(),
            code: Vec::new(),
            code_usage: 0,
        }
    }


    // See the documentation for the `KernelPages` type
    // Ignore when you not reach this part

    /// Initialize kernel vm
    ///
    /// NOTE: this function should only be called by the first process
    // pub fn init_kernel_vm(mut self, pages: &KernelPages) -> Self {
    //     // FIXME: record kernel code usage
    //     self.code = /* The kernel pages */;
    //     self.code_usage = /* The kernel code usage */;

    //     self.stack = Stack::kstack();

    //     // ignore heap for kernel process as we don't manage it

    //     self
    // }

    pub fn init_kernel_vm(mut self, pages: &KernelPages) -> Self {
        // FIXME: load `self.code` and `self.code_usage` from `pages`

        // FIXME: init kernel stack (impl the const `kstack` function)
        //        `pub const fn kstack() -> Self`
        //         use consts to init stack, same with kernel config
        self.stack = Stack::kstack();
        self.code = pages.iter().cloned().collect();
        self.code_usage = pages.iter().map(|r| r.size()).sum::<u64>();

        self
    }

    pub fn brk(&self, addr: Option<VirtAddr>) -> Option<VirtAddr> {
        self.heap.brk(
            addr,
            &mut self.page_table.mapper(),
            &mut get_frame_alloc_for_sure(),
        )
    }

    pub fn load_elf(&mut self, elf: &ElfFile) {
        let mapper = &mut self.page_table.mapper();

        let alloc = &mut *get_frame_alloc_for_sure();

        self.load_elf_code(elf, mapper, alloc);
        self.stack.init(mapper, alloc);
    }

    fn load_elf_code(&mut self, elf: &ElfFile, mapper: MapperRef, alloc: FrameAllocatorRef) {
        // FIXME: make the `load_elf` function return the code pages
        self.code =
            elf::load_elf(elf, *PHYSICAL_OFFSET.get().unwrap(), mapper, alloc, true).unwrap();
            // 返回类型：Result<Vec<PageRangeInclusive>, UnmapError>
            // 其中 Vec<PageRangeInclusive> 是加载的代码页范围
            // 例如: [PageRangeInclusive { start: Page { number: 0x0000_0000 }, end: Page { number: 0x0000_0001 } }]
            // 这里的代码页范围是从 ELF 文件中加载的代码段

        // FIXME: calculate code usage
        self.code_usage = /* The code usage */
            self.code.iter().map(|r| r.size()).sum::<u64>();
        trace!("Code usage: {}", self.code_usage);
    }

    pub fn fork(&self, stack_offset_count: u64) -> Self {
        let owned_page_table = self.page_table.fork();
        let mapper = &mut owned_page_table.mapper();

        let alloc = &mut *get_frame_alloc_for_sure();

        Self {
            page_table: owned_page_table,
            stack: self.stack.fork(mapper, alloc, stack_offset_count),
            heap: self.heap.fork(),

            // do not share code info
            code: Vec::new(),
            code_usage: 0,
        }
    }

    pub fn handle_page_fault(&mut self, addr: VirtAddr) -> bool {
        // info!("Handling page fault at address: {:#x}", addr);
        let mapper = &mut self.page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();
        // info!("Handling page fault for stack: {:#x}", addr);

        self.stack.handle_page_fault(addr, mapper, alloc)
    }

    pub(super) fn memory_usage(&self) -> u64 {
        self.stack.memory_usage() + self.heap.memory_usage() + self.code_usage
    }

    pub(super) fn clean_up(&mut self) -> Result<(), UnmapError> {
        let mapper = &mut self.page_table.mapper();
        let dealloc = &mut *get_frame_alloc_for_sure();

        // statistics for logging and debugging
        // NOTE: you may need to implement `frames_recycled` by yourself
        let start_count = dealloc.frames_recycled();

        // FIXME: implement the `clean_up` function for `Stack`
        self.stack.clean_up(mapper, dealloc)?;

        if self.page_table.using_count() == 1 {
            // free heap
            // FIXME: implement the `clean_up` function for `Heap`
            self.heap.clean_up(mapper, dealloc)?;

            // free code
            for page_range in self.code.iter() {
                let start_addr = page_range.start.start_address().as_u64();
                let page_count = page_range.count() as u64;
                let range_start = Page::containing_address(VirtAddr::new(start_addr));
                let range_end = range_start + page_count;
                let page_range = Page::range(range_start, range_end);
                unsafe {
                    // unmap the code pages
                    elf::unmap_range(
                        mapper, 
                        dealloc, 
                        // page_range.clone(),
                        page_range,
                        true
                    )?;
                }
            }

            unsafe {
                // free P1-P3
                mapper.clean_up(dealloc);

                // free P4
                dealloc.deallocate_frame(self.page_table.reg.addr);
            }
        }

        // statistics for logging and debugging
        let end_count = dealloc.frames_recycled();

        // NOTE: maybe print how many frames are recycled
        //       **you may need to add some functions to `BootInfoFrameAllocator`**

        debug!(
            "Recycled {}({:.3} MiB) frames, {}({:.3} MiB) frames in total.",
            end_count - start_count,
            ((end_count - start_count) * 4) as f32 / 1024.0,
            end_count,
            (end_count * 4) as f32 / 1024.0
        );

        Ok(())
    }

    
    // pub fn brk(&self, addr: Option<VirtAddr>) -> Option<VirtAddr> {
    //     self.heap.brk(addr,&mut self.page_table.mapper(),&mut get_frame_alloc_for_sure())
    // }
}

impl Drop for ProcessVm {
    fn drop(&mut self) {
        if let Err(err) = self.clean_up() {
            error!("Failed to clean up process memory: {:?}", err);
        }
    }
}

impl core::fmt::Debug for ProcessVm {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (size, unit) = humanized_size(self.memory_usage());

        f.debug_struct("ProcessVm")
            .field("stack", &self.stack)
            .field("heap", &self.heap)
            .field("memory_usage", &format!("{} {}", size, unit))
            .field("page_table", &self.page_table)
            .finish()
    }
}



// use alloc::format;
// use x86_64::{
//     structures::paging::{page::*, *},
//     VirtAddr,
// };
// use xmas_elf::ElfFile;

// use crate::{humanized_size, memory::*, proc::vm::stack::Stack};

// pub mod stack;
// use crate::proc::KERNEL_PID;
// use self::stack::*;

// use super::{PageTableContext, ProcessId};

// type MapperRef<'a> = &'a mut OffsetPageTable<'static>;
// type FrameAllocatorRef<'a> = &'a mut BootInfoFrameAllocator;

// pub struct ProcessVm {
//     // page table is shared by parent and child
//     pub(super) page_table: PageTableContext,

//     // stack is pre-process allocated
//     pub(super) stack: Stack,
// }

// impl ProcessVm {
//     pub fn new(page_table: PageTableContext) -> Self {
//         Self {
//             page_table,
//             stack: Stack::empty(),
//         }
//     }

//     pub fn init_kernel_vm(mut self) -> Self {
//         // TODO: record kernel code usage
//         self.stack = Stack::kstack();
//         self
//     }

//     // pub fn init_proc_stack(&mut self, pid: ProcessId) -> VirtAddr {
//     //     // FIXME: calculate the stack for pid
//     //     info!("Init process stack for pid: {}", pid);
//     //     let stack_top_addr = STACK_INIT_TOP-((pid.0 as u64 -1) * 0x1_0000_0000);
//     //     let stack_bot_addr = STACK_INIT_BOT-((pid.0 as u64 -1) * 0x1_0000_0000);
//     //     info!("Stack top addr: {:#x}", stack_top_addr);
//     //     let page_table = &mut self.page_table.mapper();
//     //     // info!("Stack top addr: {:#x}", stack_top_addr);
//     //     let alloc = &mut *get_frame_alloc_for_sure();
//     //     // info!("Mapping stack for pid: {}", pid);
//     //     let is_user_access = pid != KERNEL_PID;
//     //     info!("stack_bot_addr: {:#x}", stack_bot_addr);
//     //     match elf::map_range(
//     //         stack_bot_addr,
//     //         1,
//     //         page_table,
//     //         alloc,
//     //         is_user_access,
//     //         // false,
//     //     ){
//     //         Ok(range) => {
//     //             trace!("Stack range: {:#?}", range);
//     //         }
//     //         Err(e) => {
//     //             panic!("Failed to map stack: {:#?}", e);
//     //         }
//     //     }
//     //     self.stack = Stack::new(
//     //         Page::containing_address(VirtAddr::new(stack_top_addr)),
//     //         STACK_DEF_PAGE,
//     //     );
//     //     info!("Stack: {:#?}", self.stack);
//     //     VirtAddr::new(stack_top_addr)
//     // }

//     pub fn handle_page_fault(&mut self, addr: VirtAddr) -> bool {
//         let mapper = &mut self.page_table.mapper();
//         let alloc = &mut *get_frame_alloc_for_sure();

//         self.stack.handle_page_fault(addr, mapper, alloc)
//     }

//     pub(super) fn memory_usage(&self) -> u64 {
//         self.stack.memory_usage()
//     }

//     pub fn load_elf(
//         &mut self,
//         elf: &ElfFile
//     ) {
//         let mapper = &mut self.page_table.mapper();
//         let alloc = &mut *get_frame_alloc_for_sure();

//         // init stack
//         self.stack.init(mapper, alloc);
//         // match elf::load_elf(
//         //     elf,
//         //     *PHYSICAL_OFFSET.get().unwrap(),
//         //     mapper,
//         //     alloc,
//         //     true
//         // ) {
//         //     Ok(_) => {
//         //         info!("Process Loaded: {:#?}", elf);
//         //     }
//         //     Err(e) => {
//         //         error!("Failed to load ELF file: {:?}", e);
//         //         panic!("Failed to load ELF file: {:?}", e);
//         //     }
//         // }
//         elf::load_elf(elf, PHYSICAL_OFFSET.get().cloned().unwrap(), mapper, alloc,true).unwrap();
//     }

//     pub fn fork(&self, stack_offset_count: u64) -> Self {
//         // clone the page table context (see instructions)
//         let owned_page_table = self.page_table.fork();

//         let mapper = &mut owned_page_table.mapper();
//         let alloc = &mut *get_frame_alloc_for_sure();

//         Self {
//             page_table: owned_page_table,
//             stack: self.stack.fork(mapper, alloc, stack_offset_count),
//         }
//     }
// }

// impl core::fmt::Debug for ProcessVm {
//     fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
//         let (size, unit) = humanized_size(self.memory_usage());

//         f.debug_struct("ProcessVm")
//             .field("stack", &self.stack)
//             .field("memory_usage", &format!("{} {}", size, unit))
//             .field("page_table", &self.page_table)
//             .finish()
//     }
// }