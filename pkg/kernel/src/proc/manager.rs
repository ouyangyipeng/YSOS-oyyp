use super::*;
use crate::memory::{
    self,
    allocator::{ALLOCATOR, HEAP_SIZE},
    get_frame_alloc_for_sure, PAGE_SIZE,
};
use crate::{proc::vm::ProcessVm};
use alloc::{collections::*, format, string::String, sync::Arc, sync::Weak};
use spin::{Mutex, RwLock};
use crate::utils::humanized_size;
use vm::stack::STACK_INIT_TOP;

pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>, app_list: boot::AppListRef) {

    // FIXME: set init process as Running

    // init.write().resume();
    init.write().pause();
    debug!("Should resume running: {:#?}", init);

    // FIXME: set processor's current pid to init's pid
    // processor::print_processors();
    processor::set_pid(init.pid());
    processor::print_processors();

    PROCESS_MANAGER.call_once(|| ProcessManager::new(init, app_list));
    info!("Process Manager initialized");
    debug!("Process Manager: {:#?}", PROCESS_MANAGER.get().unwrap().processes.read());
    debug!("Process Manager: {:#?}", PROCESS_MANAGER.get().unwrap().ready_queue.lock());
    debug!("Process Manager: {:#?}", PROCESS_MANAGER.get().unwrap().ready_queue.lock().len());
    debug!("Process Manager: {:#?}", PROCESS_MANAGER.get().unwrap().ready_queue.lock().is_empty());
}

pub fn get_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

pub struct ProcessManager {
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
    app_list: boot::AppListRef,
    wait_queue: Mutex<BTreeMap<ProcessId, BTreeSet<ProcessId>>>,
}

impl ProcessManager {
    pub fn new(init: Arc<Process>, app_list: boot::AppListRef) -> Self {
        let mut processes = BTreeMap::new();
        let ready_queue = VecDeque::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);
        Self {
            processes: RwLock::new(processes),
            ready_queue: Mutex::new(ready_queue),
            app_list: app_list,
            wait_queue: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn app_list(&self) -> boot::AppListRef {
        self.app_list
    }

    #[inline]
    pub fn push_ready(&self, pid: ProcessId) {
        self.ready_queue.lock().push_back(pid);
    }

    #[inline]
    fn add_proc(&self, pid: ProcessId, proc: Arc<Process>) {
        self.processes.write().insert(pid, proc);
    }

    #[inline]
    pub fn get_proc(&self, pid: &ProcessId) -> Option<Arc<Process>> {
        self.processes.read().get(pid).cloned()
    }

    pub fn current(&self) -> Arc<Process> {
        self.get_proc(&processor::get_pid())
            .expect("No current process")
    }

    pub fn read(&self, fd: u8, buf: &mut [u8]) -> isize {
        self.current().read().read(fd, buf)
    }

    pub fn write(&self, fd: u8, buf: &[u8]) -> isize {
        self.current().read().write(fd, buf)
    }

    pub fn save_current(&self, context: &ProcessContext) {
        // FIXME: update current process's tick count
        // FIXME: save current process's context
        let cur_pid = processor::get_pid(); // 从处理器获取当前进程的pid
        if let Some(cur_proc) = self.get_proc(&cur_pid) {
            // info!("Process #{} found.", cur_pid);
            let mut cur_inner= cur_proc.write();
            // 更新运行时间
            cur_inner.tick();
            // info!("Process #{} ticks: {}", cur_pid, cur_inner.ticks_passed());
            // 保存当前进程的上下文
            cur_inner.save(context);
            // drop(cur_inner);
            // 更新当前进程的状态 → 这个部分在save里面实现了
            // if cur_inner.status() == ProgramStatus::Running {
            //     // cur_inner.set_status(ProgramStatus::Ready);
            //     cur_inner.pause();
            // }
        } else {
            warn!("Process #{} not found.", cur_pid);
        }
    }

    pub fn switch_next(&self, context: &mut ProcessContext) -> ProcessId {
        let mut queue = self.ready_queue.lock();
        if queue.is_empty() {
            warn!("No process in ready queue.");
            return processor::get_pid();
        }

        // FIXME: fetch the next process from ready queue

        // FIXME: check if the next process is ready,
        //        continue to fetch if not ready
        let next_pid = loop {
            if let Some(pid) = queue.pop_front() {
                if let Some(proc) = self.get_proc(&pid) {
                    if proc.read().status() == ProgramStatus::Ready {
                        // info!("Process #{} is ready.", pid);
                        break pid;
                    } else {
                        if proc.read().status() != ProgramStatus::Dead {
                            queue.push_back(pid);
                            warn!("Process #{} is not ready.", pid);
                        } else {
                            warn!("Process #{} is dead.", pid);
                        }
                    }
                } else {
                    warn!("Process #{} not found.", pid);
                }
            } else {
                warn!("No process in ready queue.");
                return processor::get_pid();
            }
        };
        drop(queue);
        trace!("Switch to process #{}", next_pid);

        // FIXME: restore next process's context
        if let Some(next_proc) = self.get_proc(&next_pid) {
            next_proc.write().restore(context);
        } else {
            warn!("Process #{} not found.", next_pid);
        }

        // FIXME: update processor's current pid
        processor::set_pid(next_pid);
        // processor::print_processors();

        // FIXME: return next process's pid
        next_pid
    }

    // ! discarded code in 0x04
    // pub fn spawn_kernel_thread(
    //     &self,
    //     entry: VirtAddr,
    //     name: String,
    //     proc_data: Option<ProcessData>,
    // ) -> ProcessId {
    //     let kproc = self.get_proc(&KERNEL_PID).unwrap();
    //     let page_table = kproc.read().clone_page_table();
    //     let proc_vm = Some(ProcessVm::new(page_table));
    //     let proc = Process::new(name, Some(Arc::downgrade(&kproc)), proc_vm, proc_data);
    //     let pid = proc.pid();
    //     // alloc stack for the new process base on pid
    //     let stack_top = proc.alloc_init_stack();
    //     // FIXME: set the stack frame
    //     proc.write().init_stack_frame(entry, stack_top);
    //     // FIXME: add to process map
    //     self.add_proc(pid, proc);
    //     trace!("Spawn process #{}", pid);
    //     // FIXME: push to ready queue
    //     self.push_ready(pid);
    //     trace!("Push process #{} to ready queue", pid);
    //     // FIXME: return new process pid
    //     info!("Spawn process #{} with stack top {:#x}", pid, stack_top);
    //     pid
    // }

    pub fn spawn(
        &self,
        elf: &ElfFile,
        name: String,
        parent: Option<Weak<Process>>,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc_vm = Some(ProcessVm::new(page_table));
        // info!("ProcessVm: {:#?}", proc_vm);
        let proc = Process::new(name, parent, proc_vm, proc_data);
        // info!("Process: {:#?}", proc);
        let pid = proc.pid();

        // let mut inner = proc.write();
        // info!("Stack top");
        let entry = VirtAddr::new(elf.header.pt2.entry_point());
        // FIXME: load elf to process pagetable
        // info!("Load ELF: {:#?}", elf);
        // inner.load_elf(elf);
        proc.write().load_elf(elf);
        // info!("Load ELF done");
        // FIXME: alloc new stack for process
        // inner.init_stack_frame(entry, stack_top);
        
        // let stack_top = proc.alloc_init_stack();
        proc.write().init_stack_frame(entry, VirtAddr::new(STACK_INIT_TOP));
        // proc.write().init_stack_frame(entry,  VirtAddr::new_truncate(STACK_INIT_TOP));
        // FIXME: mark process as ready
        // inner.pause();
        proc.write().pause();

        // 内存使用量
        // let proc_vm = inner.vm_mut();
        // let memory_usage = proc_vm.memory_usage();
        // inner.data_mut().set_memory_usage(memory_usage);

        // 其他data可以在这里加

        // drop(inner);

        // info!("New {:#?}", &proc);

        // FIXME: something like kernel thread
        self.add_proc(pid, proc);
        self.push_ready(pid);
        debug!("Push process #{} to ready queue", pid);

        pid 
    }

    pub fn kill_current(&self, mut ret: isize) {
        self.kill(processor::get_pid(), ret);
    }

    pub fn handle_page_fault(&self, addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
        // FIXME: handle page fault
        if err_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION){
            warn!("Page fault: protection violation at {:#x}", addr);
            return false;
        }else if err_code.contains(PageFaultErrorCode::MALFORMED_TABLE) {
            warn!("Page fault: malformed table at {:#x}", addr);
            return false;
        }else if err_code.contains(PageFaultErrorCode::INSTRUCTION_FETCH) {
            warn!("Page fault: instruction fetch at {:#x}", addr);
            return false;
        }else if err_code.contains(PageFaultErrorCode::PROTECTION_KEY) {
            warn!("Page fault: protection key at {:#x}", addr);
            return false;
        }else if err_code.contains(PageFaultErrorCode::INSTRUCTION_FETCH) {
            warn!("Page fault: instruction fetch at {:#x}", addr);
            return false;
        }else if err_code.contains(PageFaultErrorCode::PROTECTION_KEY) {
            warn!("Page fault: protection key at {:#x}", addr);
            return false;
        }else if err_code.contains(PageFaultErrorCode::SHADOW_STACK) {
            warn!("Page fault: shadow stack at {:#x}", addr);
            return false;
        }else if err_code.contains(PageFaultErrorCode::SGX) || 
                err_code.contains(PageFaultErrorCode::RMP) {
            warn!("Page fault: SGX/RMP violation at {:#x}", addr);
            return false;
        }else{
            let cur_proc = self.current();
            let cur_pid = cur_proc.pid();
            if cur_pid == KERNEL_PID {
                info!("Page fault on kernel at {:#x}", addr);
            }
            let mut cur_inner = cur_proc.write();
            let vm = cur_inner.vm_mut();
            if vm.handle_page_fault(addr) {
                return true;
            } else {
                warn!("Page fault: failed to handle page fault at {:#x}", addr);
                return false;
            }
        }
        // let curr_proc = get_process_manager().current();
        // if !err_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION)
        //     && !err_code.contains(PageFaultErrorCode::CAUSED_BY_WRITE)
        // {
        //     return false;
        // }
        // // handle page fault in current process
        // let ret = curr_proc.write().vm_mut().handle_page_fault(addr);
        // ret
    }

    pub fn kill_self(&self, ret: isize) {
        let pid = processor::get_pid();
        if pid == KERNEL_PID {
            warn!("Kernel process cannot be killed.");
            panic!("trying to kill kernel")
        } else {
            self.kill(pid, ret);
        }
    }

    pub fn kill(&self, pid: ProcessId, mut ret: isize) {
        let proc = self.get_proc(&pid);

        if proc.is_none() {
            warn!("Process #{} not found.", pid);
            return;
        }

        let proc = proc.unwrap();

        if proc.read().status() == ProgramStatus::Dead {
            warn!("Process #{} is already dead.", pid);
            return;
        }

        trace!("Kill {:#?}", &proc);

        proc.kill(ret);

        if let Some(pids) = self.wait_queue.lock().remove(&pid) {
            for pid in pids {
                self.wake_up(pid, Some(ret));
            }
        }
    }

    pub fn print_process_list(&self) {
        // let mut output = String::from("  PID | PPID | Process Name |  Ticks  | Status\n");

        // self.processes
        //     .read()
        //     .values()
        //     .filter(|p| p.read().status() != ProgramStatus::Dead)
        //     .for_each(|p| output += format!("{}\n", p).as_str());

        // let mut output =
        //     String::from("  PID | PPID | Process Name |  Ticks  | Status | Memory Usage\n");

        // for (_, p) in self.processes.read().iter() {
        //     if p.read().status() != ProgramStatus::Dead {
        //         output += format!("{}\n", p).as_str();
        //     }
        // }

        // // TODO: print memory usage of kernel heap

        // output += format!("Queue  : {:?}\n", self.ready_queue.lock()).as_str();

        // output += &processor::print_processors();

        // print!("{}", output);
        
        let mut output =
            String::from("  PID | PPID | Process Name |  Ticks  | Memory Usage | Status\n");

        for (_, p) in self.processes.read().iter() {
            if p.read().status() != ProgramStatus::Dead {
                output += format!("{}\n", p).as_str();
            }
        }

        // TODO: print memory usage of kernel heap
        let alloc: spin::MutexGuard<'_, memory::BootInfoFrameAllocator> = get_frame_alloc_for_sure();
        let frames_used = alloc.frames_used();
        let frames_recycled = alloc.frames_recycled();
        let frames_total = alloc.frames_total();
        let used = (frames_used - frames_recycled) * PAGE_SIZE as usize;
        let total = frames_total * PAGE_SIZE as usize;
        output += &Self::format_usage("Memory", used, total);
        drop(alloc);
        output += format!("Queue  : {:?}\n", self.ready_queue.lock()).as_str();
        output += &processor::print_processors();

        print!("{}", output);
    }

    // A helper function to format memory usage
    fn format_usage(name: &str, used: usize, total: usize) -> String {
        let (used_float, used_unit) = humanized_size(used as u64);
        let (total_float, total_unit) = humanized_size(total as u64);

        format!(
            "{:<6} : {:>6.*} {:>3} / {:>6.*} {:>3} ({:>5.2}%)\n",
            name,
            2,
            used_float,
            used_unit,
            2,
            total_float,
            total_unit,
            used as f32 / total as f32 * 100.0
        )
    }

    pub fn fork(&self) {
        // FIXME: get current process
        let current = self.current();

        // FIXME: fork to get child
        let child = current.fork();

        // FIXME: add child to process list
        // self.add_proc(child.pid(), child);
        // self.push_ready(child.pid()); // 换到这里来  // 不是，为啥这样会报错啊？好像是被借走了
        let child_pid = child.pid();
        self.add_proc(child_pid, child);
        self.push_ready(child_pid); // 这样又可以了？
        

        // FOR DBG: maybe print the process ready queue?
        debug!("Process ready queue: {:#?}", self.ready_queue.lock());
    }

    /// Block the process with the given pid
    pub fn block(&self, pid: ProcessId) {
        if let Some(proc) = self.get_proc(&pid) {
            // FIXME: set the process as blocked
            proc.write().block();
        }
    }

    pub fn get_exit_code(&self, pid: ProcessId) -> Option<isize> {
        if let Some(proc) = self.get_proc(&pid) {
            if proc.read().status() == ProgramStatus::Dead {
                return proc.read().exit_code();
            }
        }
        None
    }

    pub fn wait_pid(&self, pid: ProcessId) {
        let mut wait_queue = self.wait_queue.lock();
        // FIXME: push the current process to the wait queue
        //        `processor::get_pid()` is waiting for `pid`
        // 这里要先获得这个btreemap的锁，然后里面的键值是传入的这个pid
        // 而它的btreeset里面要放进我们当前的这个process的pid
        let entry_set_of_pid = wait_queue
            .entry(pid)
            .or_insert_with(BTreeSet::new);
        entry_set_of_pid.insert(processor::get_pid());
        drop(wait_queue); // 释放锁
    }

    /// Wake up the process with the given pid
    ///
    /// If `ret` is `Some`, set the return value of the process
    pub fn wake_up(&self, pid: ProcessId, ret: Option<isize>) {
        if let Some(proc) = self.get_proc(&pid) {
            let mut inner = proc.write();
            if let Some(ret) = ret {
                // FIXME: set the return value of the process
                //        like `context.set_rax(ret as usize)`
                inner.set_rax(ret as usize);
            }
            // FIXME: set the process as ready
            inner.pause();
            // FIXME: push to ready queue
            self.push_ready(pid);
        }
    }

    pub fn sem_new(&self, key: u32, value: usize) -> bool {
        trace!("Sem New: <{:#x}>", key);
        let ret = self.current().write().sem_new(key, value);
        ret
    }

    pub fn sem_remove(&self, key: u32) -> bool {
        trace!("Sem Remove: <{:#x}>", key);
        let ret = self.current().write().sem_remove(key);
        ret
    }

    pub fn sem_wait(&self, key: u32, pid: ProcessId) -> SemaphoreResult {
        trace!("Sem Wait: <{:#x}>", key);
        let ret = self.current().write().sem_wait(key, pid);
        ret
    }

    pub fn sem_signal(&self, key: u32) -> SemaphoreResult {
        trace!("Sem Signal: <{:#x}>", key);
        let ret = self.current().write().sem_signal(key);
        ret
    }

    pub fn open_file(&self, path: &str) -> u8 {
        self.current().write().open_file(path)
    }
    
    pub fn close_file(&self, fd: u8) -> bool {
        self.current().write().close_file(fd)
    }
}

impl core::fmt::Debug for ProcessManager {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ProcessManager {{ ... }}")
    }
}
impl core::fmt::Display for ProcessManager {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ProcessManager {{ ... }}")
    }
}