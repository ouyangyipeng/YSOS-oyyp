use core::ptr::addr_of_mut;
use lazy_static::lazy_static;
use x86_64::registers::segmentation::Segment;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;  // 双重故障栈索引:栈0用于双重故障（最严重的CPU异常）
pub const PAGE_FAULT_IST_INDEX: u16 = 1;    // 页故障栈索引:栈1用于页故障（需处理缺页异常）

// 还要处理ssf，gpf，mc等异常
pub const STACK_SEGMENT_IST_INDEX: u16 = 2; // 栈段故障栈索引：栈2用于栈段故障
pub const GPF_IST_INDEX: u16 = 3;          // 一般保护故障栈索引：栈3用于一般保护故障
// pub const MACHINE_CHECK_IST_INDEX: u16 = 4; // 机器检查栈索引：栈4用于机器检查异常（这个不一定要）
// pub const NMI_IST_INDEX: u16 = 5;         // 非屏蔽中断栈索引：栈5用于非屏蔽中断
pub const TIMER_IST_INDEX: u16 = 2;       // 定时器栈索引：栈6用于定时器中断

pub const IST_SIZES: [usize; 8] = [0x1000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000]; // 每个IST栈大小(4KB)

// ! 核心组件
lazy_static! {// 延迟初始化复杂全局变量，避免编译期计算(上网查的这个宏)
    static ref TSS: TaskStateSegment = {// 任务状态段
        trace!("Creating TSS...");
        let mut tss = TaskStateSegment::new();

        // initialize the TSS with the static buffers
        // will be allocated on the bss section when the kernel is load
        //
        // DO NOT MODIFY THE FOLLOWING CODE
        
        // 特权级0的栈（内核模式）
        tss.privilege_stack_table[0] = {
            const STACK_SIZE: usize = IST_SIZES[0];
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];// 在BSS段静态分配4KB栈空间
            let stack_start = VirtAddr::from_ptr(addr_of_mut!(STACK));// 计算栈顶地址（x86栈从高地址向低地址增长）
            let stack_end = stack_start + STACK_SIZE as u64;
            info!(
                "Privilege Stack  : 0x{:016x}-0x{:016x}",
                stack_start.as_u64(),
                stack_end.as_u64()
            );
            stack_end
        };


        // FIXME: fill tss.interrupt_stack_table with the static stack buffers like above
        // You can use `tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize]`

        // 中断1号栈double fault
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize]={
            const STACK_SIZE:usize=IST_SIZES[1];
            static mut STACK:[u8;STACK_SIZE]=[0;STACK_SIZE];
            let stack_start=VirtAddr::from_ptr(addr_of_mut!(STACK));
            let stack_end=stack_start+STACK_SIZE as u64;
            info!(
                "Double Fault Stack: 0x{:016x}-0x{:016x}",
                stack_start.as_u64(),
                stack_end.as_u64()
            );
            stack_end
        };

        // 中断2号栈page fault
        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = IST_SIZES[2];
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(addr_of_mut!(STACK));
            let stack_end = stack_start + STACK_SIZE as u64;
            info!(
                "Interrupt(Page Fault) Stack  : 0x{:016x}-0x{:016x}",
                stack_start.as_u64(),
                stack_end.as_u64()
            );
            stack_end
        };

        // 中断3号栈stack segment fault
        tss.interrupt_stack_table[STACK_SEGMENT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = IST_SIZES[3];
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(addr_of_mut!(STACK));
            let stack_end = stack_start + STACK_SIZE as u64;
            info!(
                "Interrupt(Stack Segment Fault) Stack  : 0x{:016x}-0x{:016x}",
                stack_start.as_u64(),
                stack_end.as_u64()
            );
            stack_end
        };

        // 中断4号栈general protection fault
        tss.interrupt_stack_table[GPF_IST_INDEX as usize] = {
            const STACK_SIZE: usize = IST_SIZES[4];
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(addr_of_mut!(STACK));
            let stack_end = stack_start + STACK_SIZE as u64;
            info!(
                "Interrupt(General Protection Fault) Stack  : 0x{:016x}-0x{:016x}",
                stack_start.as_u64(),
                stack_end.as_u64()
            );
            stack_end
        };

        // 中断5号栈machine check（可被换走，如果后面实验还有别的要？）
        // tss.interrupt_stack_table[MACHINE_CHECK_IST_INDEX as usize] = {
        //     const STACK_SIZE: usize = IST_SIZES[5];
        //     static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
        //     let stack_start = VirtAddr::from_ptr(addr_of_mut!(STACK));
        //     let stack_end = stack_start + STACK_SIZE as u64;
        //     info!(
        //         "Interrupt(Machine Check) Stack  : 0x{:016x}-0x{:016x}",
        //         stack_start.as_u64(),
        //         stack_end.as_u64()
        //     );
        //     stack_end
        // };

        // 中断6号栈nmi
        // tss.interrupt_stack_table[5] = {
        //     const STACK_SIZE: usize = IST_SIZES[6];
        //     static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
        //     let stack_start = VirtAddr::from_ptr(addr_of_mut!(STACK));
        //     let stack_end = stack_start + STACK_SIZE as u64;
        //     info!(
        //         "Interrupt(NMI) Stack  : 0x{:016x}-0x{:016x}",
        //         stack_start.as_u64(),
        //         stack_end.as_u64()
        //     );
        //     stack_end
        // };

        // 中断7号栈timer
        tss.interrupt_stack_table[TIMER_IST_INDEX as usize] = {
            const STACK_SIZE: usize = IST_SIZES[6];
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(addr_of_mut!(STACK));
            let stack_end = stack_start + STACK_SIZE as u64;
            info!(
                "Interrupt(Timer) Stack  : 0x{:016x}-0x{:016x}",
                stack_start.as_u64(),
                stack_end.as_u64()
            );
            stack_end
        };

        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, KernelSelectors) = {// GDT表及选择
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.append(Descriptor::kernel_code_segment());// 内核代码段（可执行）
        let data_selector = gdt.append(Descriptor::kernel_data_segment());// 内核数据段（不可执行可读写）
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));// 任务状态段描述符
        (
            gdt,
            KernelSelectors {
                code_selector,
                data_selector,
                tss_selector,
            },
        )
    };
}

#[derive(Debug)]
// ! 选择子封装结构
pub struct KernelSelectors {
    pub code_selector: SegmentSelector, // 代码段选择子（供内存管理使用）
    pub data_selector: SegmentSelector, // 数据段选择子（供内存管理使用）
    tss_selector: SegmentSelector,    // 任务状态段选择子（不直接暴露，这个只给内部加载用）
}

// ! 初始化函数
pub fn init() {
    use x86_64::instructions::segmentation::{CS, DS, ES, FS, GS, SS};
    use x86_64::instructions::tables::load_tss;
    use x86_64::PrivilegeLevel;

    trace!("Initializing GDT...");

    GDT.0.load();// 加载GDT到GDTR寄存器，lgdt指令，在实验报告有写，开始前预习了
    trace!("GDT Loaded.");
    trace!("Loading TSS...");
    unsafe {
        // 设置段寄存器
        CS::set_reg(GDT.1.code_selector);// 设置代码段选择子
        DS::set_reg(GDT.1.data_selector);// 设置数据段选择子
        SS::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));// 设置栈段选择子
        ES::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
        FS::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
        GS::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
        load_tss(GDT.1.tss_selector);// 加载TSS
    }
    trace!("TSS Loaded.");

    // 统计IST总大小
    let mut size = 0;

    for &s in IST_SIZES.iter() {
        size += s;
    }

    let (size, unit) = crate::humanized_size(size as u64);
    trace!("Kernel IST Size  : {:>7.*} {}", 3, size, unit);

    info!("GDT Initialized.");
}

// ! 选择子获取接口
pub fn get_selector() -> &'static KernelSelectors {
    &GDT.1
}

// ! 用户态选择子获取接口
// pub fn get_user_selector() -> SegmentSelector {
//     SegmentSelector::new(0, PrivilegeLevel::Ring3)
// }
