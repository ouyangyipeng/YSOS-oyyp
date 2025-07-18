mod context;
mod data;
mod paging;
mod pid;
mod process;
mod vm;
pub mod processor;
pub mod manager;
pub mod sync;

use manager::*;
use process::*;
use crate::proc::vm::ProcessVm;
use crate::memory::PAGE_SIZE;
use alloc::sync::Arc;
use xmas_elf::ElfFile;
use alloc::string::{String, ToString};
use sync::*;

use itoa::Buffer;
// Vec
use alloc::vec::Vec;

// use alloc::string::String;
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
pub fn init(boot_info: &'static boot::BootInfo) {
    /* 将内核包装成进程，并将其传递给 ProcessManager，使其成为第一个进程 */
    let proc_vm = ProcessVm::new(PageTableContext::new()).init_kernel_vm(&boot_info.kernel_pages);

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
    // kproc_data.set_memory_usage(proc_vm.memory_usage());

    // kernel process
    let kproc = Process::new(
        String::from("kernel"),
        None,
        Some(proc_vm),
        Some(kproc_data),
    );
    // kproc.write().resume();
    let app_list = boot_info.loaded_apps.as_ref();
    manager::init(kproc, app_list);
    manager::get_process_manager().print_process_list();
}

pub fn switch(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // FIXME: switch to the next process
        //      - save current process's context

        // 输出当前进程的context
        trace!("Current process context: {:#?}", context);
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
        trace!("Switch from {} to {}", pid, next_pid);
    });
}

// ! discarded code in 0x04
// pub fn spawn_kernel_thread(entry: fn() -> !, name: String, data: Option<ProcessData>) -> ProcessId {
//     x86_64::instructions::interrupts::without_interrupts(|| {
//         let entry = VirtAddr::new(entry as usize as u64);
//         get_process_manager().spawn_kernel_thread(entry, name, data)
//     })
// }

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

pub fn list_app() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let app_list = get_process_manager().app_list();
        if app_list.is_none() {
            println!("[!] No app found in list!");
            return;
        }

        let apps = app_list
            .unwrap()
            .iter()
            .map(|app| app.name.as_str())
            .collect::<Vec<&str>>()
            .join(", ");

        // TODO: print more information like size, entry point, etc.

        println!("[+] App list:");
        for app in app_list.unwrap() {
            println!("[+] App: {}", app.name.as_str());
            
            // 打印完整的 ELF 头信息
            // println!("{}", app.elf.header);
            
            // 添加额外的格式化信息
            // println!("    ELF Class:        {:?}", app.elf.header.pt1.class());
            // println!("    Data Encoding:    {:?}", app.elf.header.pt1.data());
            // println!("    OS/ABI:           {:?}", app.elf.header.pt1.os_abi());
            
            // 使用 HeaderPt2 的 getter 方法获取统一的值
            println!("    Entry Point:      0x{:016X}", app.elf.header.pt2.entry_point());
            println!("    Program Headers:  {} entries, {} bytes each",
                    app.elf.header.pt2.ph_count(),
                    app.elf.header.pt2.ph_entry_size());
            println!("    Section Headers:  {} entries, {} bytes each",
                    app.elf.header.pt2.sh_count(),
                    app.elf.header.pt2.sh_entry_size());
            
            // 计算文件大小估计 (这只是粗略估计)
            // let estimated_size = app.elf.header.pt2.ph_offset() as u64 + 
            //                     (app.elf.header.pt2.ph_count() as u64 * app.elf.header.pt2.ph_entry_size() as u64);
            // println!("    Estimated Size:   ~{} bytes", estimated_size);
            
            println!("----------------------------------------");
        }
        println!("[+] Total {} apps", app_list.unwrap().len());
        println!("[+] App list end.");
    });
}

pub fn spawn(name: &str) -> Option<ProcessId> {
    let app = x86_64::instructions::interrupts::without_interrupts(|| {
        let app_list = get_process_manager().app_list()?;
        app_list.iter().find(|&app| app.name.eq(name))
    })?;
    // info!("Found app: {}", name);

    elf_spawn(name.to_string(), &app.elf)
}

pub fn elf_spawn(name: String, elf: &ElfFile) -> Option<ProcessId> {
    let pid = x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let process_name = name.to_lowercase();
        let parent = Arc::downgrade(&manager.current());
        // info!("Spawning process: {}", process_name);
        let pid = manager.spawn(elf, name, Some(parent), None);

        debug!("Spawned process: {}#{}", process_name, pid);
        pid
    });

    Some(pid)
}

pub fn read(fd: u8, buf: &mut [u8]) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().read(fd, buf))
}

pub fn write(fd: u8, buf: &[u8]) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().write(fd, buf))
}

pub fn exit(ret: isize, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        // FIXME: implement this for ProcessManager
        manager.kill_self(ret);
        // info!("Process {} exited with code {}", manager.current().read().name(), ret);
        manager.switch_next(context);
        // info!("Process {} switched", manager.current().read().name());
    })
}

pub fn wait_process(pid: ProcessId, context: &mut ProcessContext){
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        if let Some(ret) = manager.get_exit_code(pid) {
            context.set_rax(ret as usize);
        } else {
            manager.wait_pid(pid);
            manager.save_current(context);
            manager.current().write().block();
            manager.switch_next(context);
        }
    })
}

// pub fn wait_process(pid: ProcessId, context: &mut ProcessContext){
//     x86_64::instructions::interrupts::without_interrupts(|| {
//         let proc = get_process_manager().get_proc(&pid).unwrap();
//         // if !still_alive(pid) {
//         //     let exit_code = proc.read().exit_code().unwrap();
//         //     context.set_rax(exit_code as usize); 
//         //     info!("Process {} exited with code {}", pid, exit_code);
//         //     get_process_manager().save_current(context);
//         //     get_process_manager().switch_next(context);
//         // }
//         info!("now proc: {}", proc.read().name());
//         if let Some(ret) = proc.read().exit_code() {
//             context.set_rax(ret as usize);
//             info!("Process {} exited with code {}", pid, ret);
//             info!("now proc: {}", proc.read().name());
//             get_process_manager().save_current(context);
//             get_process_manager().switch_next(context);
//         } else if pid == KERNEL_PID {
//             // kernel process
//             context.set_rax(0);
//             info!("Kernel process {}", pid);
//         } else if proc.read().status() == ProgramStatus::Dead {
//             // process is dead
//             context.set_rax(0);
//             info!("Process {} is dead", pid);
//         } else {
//             // process is still alive
//             // info!("Process {} is still alive", pid);
//             // super::wait(pid);
//             // info!("Process {} has exited", pid);
//             // let exit_code = proc.read().exit_code().unwrap();
//             // context.set_rax(exit_code as usize);
//             // info!("Process {} exited with code {}", pid, exit_code);
//             // info!("now proc: {}", proc.read().name());
//             // get_process_manager().save_current(context);
//             // info!("now proc: {}", proc.read().name());
//             // get_process_manager().switch_next(context);
//             // info!("Process {} switched", get_process_manager().current().read().name());
//             context.set_rax(2333);
//         }
//     })
// }

#[inline]
pub fn still_alive(pid: ProcessId) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // check if the process is still alive
        match get_process_manager().get_proc(&pid) {
            Some(proc) => {
                let proc = proc.read();
                proc.status() != ProgramStatus::Dead
            }
            _ => false,
        }
    })
}

pub fn fork(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        // FIXME: save_current as parent
        // let parent = Arc::downgrade(&manager.current());
        let parent_pid = manager.current().pid();
        manager.save_current(context);
        // FIXME: fork to get child
        // let child = manager.fork();
        manager.fork();
        // FIXME: push to child & parent to ready queue
        // manager.push_ready(child); // 这里不用吧？
        manager.push_ready(parent_pid);
        // FIXME: switch to next process
        manager.switch_next(context);
    })
}

// pub fn sem_new(key: u32, value: usize) -> usize {
//     x86_64::instructions::interrupts::without_interrupts(|| {
//         get_process_manager().sem_new(key, value) as usize
//     })
// }
pub fn sem_new(key: u32, val: usize) -> usize {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        // info!("Creating new semaphore with key: {}, value: {}", key, val);
        let ret = manager.current().write().sem_new(key, val);
        // info!("returned from sem_new: {}", ret);
        ret as usize
    })
}

pub fn sem_remove(key: u32) -> usize {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().sem_remove(key) as usize
    })
}

pub fn sem_signal(key: u32, context: &mut ProcessContext){
    x86_64::instructions::interrupts::without_interrupts(|| {
        let ret = get_process_manager().sem_signal(key);
        match ret {
            SemaphoreResult::Ok => {
                context.set_rax(0); // 成功
            }
            SemaphoreResult::NotExist => {
                context.set_rax(1); // 信号量不存在
            }
            SemaphoreResult::WakeUp(pid) => {
                context.set_rax(2); // 唤醒了一个进程
                get_process_manager().wake_up(pid, None);
            }
            _ => {
                context.set_rax(usize::MAX); // 未知错误
            }
        }
    })
}

pub fn sem_wait(key: u32, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let pid = processor::get_pid();
        let ret = manager.sem_wait(key, pid);
        match ret {
            SemaphoreResult::Ok => context.set_rax(0),
            SemaphoreResult::NotExist => context.set_rax(1),
            SemaphoreResult::Block(pid) => {
                // FIXME: save, block it, then switch to next
                //        use `save_current` and `switch_next`
                manager.save_current(context);
                manager.block(pid);
                manager.switch_next(context);
            }
            _ => unreachable!(),
        }
    })
}

pub fn open_file(path: &str) -> u8 {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().open_file(path))
}

pub fn close_file(fd: u8) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().close_file(fd))
}

pub fn brk(addr: Option<VirtAddr>) -> Option<VirtAddr> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // NOTE: `brk` does not need to get write lock
        get_process_manager().current().read().brk(addr)
    })
}