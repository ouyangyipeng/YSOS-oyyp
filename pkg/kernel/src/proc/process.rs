use super::vm::stack::STACK_MAX_PAGES;
use super::*;
use crate::memory::*;
use crate::proc::vm::ProcessVm;
use alloc::sync::{Weak, Arc};
use alloc::vec::Vec;
use spin::*;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::*;
use elf::*;
use x86_64::VirtAddr;
// for Type::Load
use xmas_elf::program::Type;

#[derive(Clone)]
pub struct Process {
    pid: ProcessId,                     // 进程id，在pid.rs
    inner: Arc<RwLock<ProcessInner>>,   // 内部数据的智能指针
}

pub struct ProcessInner {
    name: String,                   // 进程名称
    parent: Option<Weak<Process>>,  // 父进程的弱引用
    children: Vec<Arc<Process>>,    // 子进程的强引用
    ticks_passed: usize,            // 进程已经运行的时钟周期数
    status: ProgramStatus,          // 进程状态（来自其他模块的枚举）
    context: ProcessContext,        // 进程上下文（寄存器等，来自context.rs）
    exit_code: Option<isize>,       // 进程退出码
    proc_data: Option<ProcessData>, // 进程数据（来自data.rs）
    proc_vm: Option<ProcessVm>,     // 进程虚拟内存管理（来自vm/mod.rs）
}

impl Process {
    #[inline]
    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<ProcessInner> {
        self.inner.write()
    }

    #[inline]
    pub fn read(&self) -> RwLockReadGuard<ProcessInner> {
        self.inner.read()
    }

    pub fn new(
        name: String,
        parent: Option<Weak<Process>>,
        proc_vm: Option<ProcessVm>,
        proc_data: Option<ProcessData>,
    ) -> Arc<Self> {
        let name = name.to_ascii_lowercase();

        // create context
        let pid = ProcessId::new();
        let proc_vm = proc_vm.unwrap_or_else(|| ProcessVm::new(PageTableContext::new()));

        let inner = ProcessInner {
            name,
            parent,
            status: ProgramStatus::Ready,
            context: ProcessContext::default(),
            ticks_passed: 0,
            exit_code: None,
            children: Vec::new(),
            proc_vm: Some(proc_vm),
            proc_data: Some(proc_data.unwrap_or_default()),
        };

        trace!("New process {}#{} created.", &inner.name, pid);

        // create process struct
        Arc::new(Self {
            pid,
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    pub fn kill(&self, mut ret: isize) {
        let mut inner = self.inner.write();

        debug!(
            "Killing process {}#{} with ret code: {}",
            inner.name(),
            self.pid,
            ret
        );

        inner.kill(self.pid, ret);
    }

    // pub fn alloc_init_stack(&self) -> VirtAddr {
    //     // info!("Allocating init stack for process");
    //     self.write().vm_mut().init_proc_stack(self.pid)
    // }

    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // FIXME: lock inner as write
        let mut inner = self.inner.write();
        // FIXME: inner fork with parent weak ref
        let parent = Arc::downgrade(self);
        // 先创建新进程child的pid
        let child_pid = ProcessId::new();
        let child_inner = inner.fork(parent);

        // FOR DBG: maybe print the child process info
        //          e.g. parent, name, pid, etc.
        info!("Forking process: parent={}, child={}, name={}", inner.name(), child_pid, child_inner.name());

        // FIXME: make the arc of child
        let child = Arc::new(Process{ // 使用self不使用process的原因是“紫禁城（doge”要直接创造一个新的进程
            pid: child_pid,
            inner: Arc::new(RwLock::new(child_inner)),
        });// 这里用强Arc对吗？
        // FIXME: add child to current process's children list
        inner.children.push(child.clone());
        // FIXME: set fork ret value for parent with `context.set_rax`
        inner.context.set_rax(child_pid.0 as usize);
        // FIXME: mark the child as ready & return it
        // inner.pause(); // 不是哥们 这里应该要把父进程的状态改成ready吧？
        // child.write().pause(); // 这样？
        child.inner.write().pause(); // 这样？
        child
    }
}

impl ProcessInner {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tick(&mut self) {
        self.ticks_passed += 1;
    }

    pub fn ticks_passed(&self) -> usize {
        self.ticks_passed
    }

    pub fn set_status(&mut self, status: ProgramStatus) {
        self.status = status;
    }

    pub fn status(&self) -> ProgramStatus {
        self.status
    }

    pub fn pause(&mut self) {
        self.status = ProgramStatus::Ready;
    }

    pub fn resume(&mut self) {
        self.status = ProgramStatus::Running;
    }

    pub fn exit_code(&self) -> Option<isize> {
        self.exit_code
    }

    pub fn clone_page_table(&self) -> PageTableContext {
        self.proc_vm.as_ref().unwrap().page_table.clone_level_4()
    }

    pub fn is_ready(&self) -> bool {
        self.status == ProgramStatus::Ready
    }

    pub fn vm(&self) -> &ProcessVm {
        self.proc_vm.as_ref().unwrap()
    }

    pub fn vm_mut(&mut self) -> &mut ProcessVm {
        // info!("vm_mut");
        self.proc_vm.as_mut().unwrap()
    }

    pub fn data(&self) -> &ProcessData {
        self.proc_data.as_ref().unwrap()
    }

    pub fn data_mut(&mut self) -> &mut ProcessData {
        self.proc_data.as_mut().unwrap()
    }

    pub fn handle_page_fault(&mut self, addr: VirtAddr) -> bool {
        self.vm_mut().handle_page_fault(addr)
    }

    // init stack frame来自context.rs
    pub fn init_stack_frame(&mut self, entry: VirtAddr, stack_top: VirtAddr) {
        self.context.init_stack_frame(entry, stack_top);
    }

    /// Save the process's context
    /// mark the process as ready
    pub(super) fn save(&mut self, context: &ProcessContext) {
        // FIXME: save the process's context
        self.context.save(context);
        if self.status == ProgramStatus::Running {
            // self.context.save(context);
            self.pause();
            // info!("Process {} is paused.", self.name);
        } else {
            warn!("Process {} is not running.", self.name);
        }
    }

    /// Restore the process's context
    /// mark the process as running
    pub(super) fn restore(&mut self, context: &mut ProcessContext) {
        // FIXME: restore the process's context
        self.context.restore(context);
        if self.status == ProgramStatus::Ready {
            // self.context.restore(context);
            // self.vm_mut().page_table.load();
            self.vm().page_table.load();
            self.resume();
        } else {
            warn!("Process {} is not ready.", self.name);
        }

        // FIXME: restore the process's page table
    }

    pub fn parent(&self) -> Option<Arc<Process>> {
        self.parent.as_ref().and_then(|p| p.upgrade())
    }

    pub fn kill(&mut self, pid:ProcessId, ret: isize) {
        // FIXME: set exit code
        self.exit_code = Some(ret);

        // FIXME: set status to dead
        self.status = ProgramStatus::Dead;
        // info!("Process {}#{} killed.{}", self.name, pid, ret);

        // FIXME: take and drop unused resources
        // self.clean_stack(pid);
        self.proc_vm.take();
        self.proc_data.take();
    }

    // fn clean_stack(&mut self, pid: ProcessId) {
    //     let page_table = self.page_table.take().unwrap();
    //     let mut mapper = page_table.mapper();
    //     let frame_deallocator = &mut *get_frame_alloc_for_sure();
    //     let start_count = frame_deallocator.recycled_count();
    //     let proc_data = self.proc_data.as_mut().unwrap();
    //     let stack = proc_data.stack_segment.unwrap();
    //     trace!(
    //         "Free stack for {}#{}: [{:#x} -> {:#x}) ({} frames)",
    //         self.name,
    //         pid,
    //         stack.start.start_address(),
    //         stack.end.start_address(),
    //         stack.count()
    //     );
    //     elf::unmap_range(
    //         stack.start.start_address().as_u64(),
    //         stack.count() as u64,
    //         &mut mapper,
    //         frame_deallocator,
    //         true,
    //     )
    //     .unwrap();
    // }

    /// elf的load elf
    // pub fn load_elf(&mut self, elf: &ElfFile) {
    //     self.vm_mut().load_elf(elf);
    // }

    pub fn load_elf(&mut self , elf: &ElfFile){
        // let mut code_pages = 0;
        // let mut code_start = None;

        // // 遍历 ELF 程序头
        // for ph in elf.program_iter() {
        //     if ph.get_type() == Ok(Type::Load) && ph.flags().is_execute() {
        //         // 记录代码段起始地址（第一个可执行段）
        //         if code_start.is_none() {
        //             code_start = Some(VirtAddr::new(ph.virtual_addr()));
        //         }

        //         // 计算该段占用的页数
        //         let page_count = ((ph.virtual_addr() + ph.mem_size() + 0xfff) & !0xfff) / 0x1000;
        //         code_pages += page_count as usize;
        //     }
        // }
        // self.data_mut().code_pages = code_pages;
        // if let Some(start) = code_start {
        //     self.data_mut().code_start = start;
        // }
        self.vm_mut().load_elf(elf)
    }

    pub fn fork(&mut self, parent: Weak<Process>) -> ProcessInner {
        // FIXME: fork the process virtual memory struct
        // 这个应该是要后算的？因为fork里面有个stack_offset_count啊
        // FIXME: calculate the real stack offset
        // 以上两个写一起，暂时认为顺序应该是反过来的。
        // 这里的逻辑应该是，按照文档的图，总子进程个数n，那就是0x400000000000-(n+1)*0x100000000？
        let child_count = self.children.len() as u64;
        // let stack_offset_count = (child_count + 1) * 0x100000000;//这不对，应该是页数
        let stack_offset_count = (child_count + 1) * STACK_MAX_PAGES;
        let child_vm = self //克隆父进程的虚拟内存空间
            .proc_vm
            .as_ref()
            .unwrap()
            .fork(stack_offset_count);

        // FIXME: update `rsp` in interrupt stack frame
        // rsp是中断栈帧的返回地址，先克隆上下文的寄存器状态
        let mut child_context = self.context.clone();
        // 然后计算新的栈顶地址，因为在vm的fork（其实是stack的fork）里面我们尝试分配新的栈空间
        // 高32位用进程新栈基地址的高32位，低32位使用原栈顶地址的低32位（保留栈偏移量）
        let child_stack_start_addr = child_vm.stack.range.start.start_address().as_u64();
        let high32 = !0xFFFFFFFF & child_stack_start_addr;
        let parent_stack_top = self.context.value.stack_frame.stack_pointer.as_u64();
        let low32 = parent_stack_top & 0xFFFFFFFF;
        let new_rsp = high32 | low32;
        let child_stack_top = VirtAddr::new(new_rsp);
        // 然后设置新的栈顶地址
        child_context.value.stack_frame.stack_pointer = child_stack_top;
        // 这样在返回的时候，子进程就可以从新的栈顶地址开始找
        // FIXME: set the return value 0 for child with `context.set_rax`
        child_context.set_rax(0);
        // FIXME: clone the process data struct
        let child_proc_data = self
            .proc_data
            .as_ref()
            .unwrap()
            .clone(); // 这里是深拷贝
        // FIXME: construct the child process inner
        // 我们克隆父进程的名字后加一个child再加上当前序号，也就是children.len() + 1
        let child_name = self.name.clone() + "-child" + &(self.children.len() + 1).to_string();
        let child_inner = ProcessInner {
            name: child_name,
            parent: Some(parent),
            children: Vec::new(),
            ticks_passed: 0,
            status: ProgramStatus::Ready,
            context: child_context,
            exit_code: None,
            proc_data: Some(child_proc_data),
            proc_vm: Some(child_vm),
        };

        // NOTE: return inner because there's no pid record in inner
        child_inner
    }
}

impl core::ops::Deref for Process {
    type Target = Arc<RwLock<ProcessInner>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl core::ops::Deref for ProcessInner {
    type Target = ProcessData;

    fn deref(&self) -> &Self::Target {
        self.proc_data
            .as_ref()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::ops::DerefMut for ProcessInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.proc_data
            .as_mut()
            .expect("Process data empty. The process may be killed.")
    }
}


impl core::fmt::Debug for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let inner = self.inner.read();
        f.debug_struct("Process")
            .field("pid", &self.pid)
            .field("name", &inner.name)
            .field("parent", &inner.parent().map(|p| p.pid))
            .field("status", &inner.status)
            .field("ticks_passed", &inner.ticks_passed)
            .field("children", &inner.children.iter().map(|c| c.pid.0))
            .field("status", &inner.status)
            .field("context", &inner.context)
            .field("vm", &inner.proc_vm)
            .finish()
    }
}

impl core::fmt::Display for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let inner = self.inner.read();
        write!(
            f,
            // " #{:-3} | #{:-3} | {:12} | {:7} | {:?} | {:?} | {:?} | {:?}",
            " #{:-3} | #{:-3} | {:12} | {:7} | {:?} | {:?}",
            self.pid.0,
            inner.parent().map(|p| p.pid.0).unwrap_or(0),
            inner.name,
            inner.ticks_passed,
            inner.status,
            inner.vm().memory_usage(),
            // inner.data().memory_usage(),
            // inner.data().code_pages(),
            // inner.data().code_start(),
        )?;
        Ok(())
    }
}
