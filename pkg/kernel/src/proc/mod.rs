mod context;
mod data;
pub mod manager;
mod paging;
mod pid;
mod process;
mod processor;
mod vm;

use manager::*;
use process::*;
use crate::proc::vm::ProcessVm;
use crate::memory::PAGE_SIZE;

use itoa::Buffer;

use alloc::string::String;
pub use context::ProcessContext;
pub use paging::PageTableContext;
pub use data::ProcessData;
pub use pid::ProcessId;

use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::VirtAddr;
pub const KERNEL_PID: ProcessId = ProcessId(1);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProgramStatus {
    Running,
    Ready,
    Blocked,
    Dead,
}

/// init process manager
pub fn init() {
    /* 将内核包装成进程，并将其传递给 ProcessManager，使其成为第一个进程 */
    let proc_vm = ProcessVm::new(PageTableContext::new()).init_kernel_vm();

    trace!("Init kernel vm: {:#?}", proc_vm);

    let mut kproc_data = ProcessData::default();
    /*
    文档：
    为了实现内核栈的自动扩容、内存统计等功能，在创建内核时需要填充内核的进程信息。
    利用定义好的内存布局、bootloader 的实现和启动配置文件的内容，将内核的信息填充到 ProcessData 中。
    */
    // kproc_data.set_env(
    //     "kernel_stack",
    //     &format!(
    //         "{:#x} - {:#x}",
    //         proc_vm.kernel_stack_start(),
    //         proc_vm.kernel_stack_end()
    //     ),
    // );
    kproc_data.set_env("kernel_version", "1.0");
    kproc_data.set_env("boot_mode", "direct");

    // 将所有内核信息以字符串形式存入环境变量
    kproc_data.set_env("kernel_stack_address", "0xFFFFFF0100000000");
    kproc_data.set_env("kernel_stack_size", "512"); // 单位：4KiB 页数
    kproc_data.set_env("physical_memory_offset", "0xFFFF800000000000");
    kproc_data.set_env("kernel_path", r"\KERNEL.ELF");
    kproc_data.set_env("kernel_stack_auto_grow", "0"); // 0=false
    kproc_data.set_env("kernel_entry_point", "0xFFFFFF0000000000"); // _start

    // 从链接脚本中提取的内存布局信息
    kproc_data.set_env(".rodata_start", "0xFFFFFF0000000000");
    kproc_data.set_env(".text_start", "0xFFFFFF0000010000");
    kproc_data.set_env(".bss_end", "0xFFFFFF0000030000");

    // // 需要扩展ProcessData的信息
    // kproc_data.kernel_stack_base = KSTACK_INIT_PAGE;
    // kproc_data.kernel_stack_size = KSTACK_DEF_PAGE;
    // let memory_usage = proc_vm.memory_usage(); // 获取内存使用量
    // kproc_data.memory_usage = memory_usage;
    kproc_data.set_env(
        "kernel_memory_usage",
        Buffer::new().format(proc_vm.memory_usage()),
    );

    // kernel process
    let kproc = Process::new(
        String::from("kernel"),
        None,
        Some(proc_vm),
        Some(kproc_data),
    );
    manager::init(kproc);
    manager::get_process_manager().print_process_list();

    info!("Process Manager Initialized.");
}

pub fn switch(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // FIXME: switch to the next process
        //      - save current process's context
        manager::get_process_manager().save_current(context);

        //      - handle ready queue update
        let pid = processor::get_pid();
        let pm= manager::get_process_manager();
        let proc = pm.current();
        if proc.read().status() != ProgramStatus::Dead {
            pm.push_ready(pid);
        }

        //      - restore next process's context
        let next_pid = manager::get_process_manager().switch_next(context);
        info!("Switch to process {}", next_pid);
    });
}

pub fn spawn_kernel_thread(entry: fn() -> !, name: String, data: Option<ProcessData>) -> ProcessId {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let entry = VirtAddr::new(entry as usize as u64);
        get_process_manager().spawn_kernel_thread(entry, name, data)
    })
}

pub fn print_process_list() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().print_process_list();
    })
}

pub fn env(key: &str) -> Option<String> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // FIXME: get current process's environment variable
        manager::get_process_manager().current().read().env(key)
    })
}

pub fn process_exit(ret: isize) -> ! {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().kill_current(ret);
    });

    loop {
        x86_64::instructions::hlt();
    }
}

pub fn handle_page_fault(addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().handle_page_fault(addr, err_code)
    })
}
