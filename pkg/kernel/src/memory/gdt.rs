use core::ptr::addr_of_mut;
use lazy_static::lazy_static;
use x86_64::registers::segmentation::Segment;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;  // 双重故障栈索引:栈0用于双重故障（最严重的CPU异常）
pub const PAGE_FAULT_IST_INDEX: u16 = 1;    // 页故障栈索引:栈1用于页故障（需处理缺页异常）

pub const IST_SIZES: [usize; 3] = [0x1000, 0x1000, 0x1000]; // 每个IST栈大小(4KB)

// ! 核心组件
lazy_static! {// 延迟初始化复杂全局变量，避免编译期计算(上网查的这个宏)
    static ref TSS: TaskStateSegment = {// 任务状态段
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

        // 特权级1的栈（内核模式）double fault
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = IST_SIZES[1];
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(addr_of_mut!(STACK));
            let stack_end = stack_start + STACK_SIZE as u64;
            info!(
                "Interrupt(Double Fault) Stack  : 0x{:016x}-0x{:016x}",
                stack_start.as_u64(),
                stack_end.as_u64()
            );
            stack_end
        };

        // 特权级2的栈（内核模式）page fault
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

    GDT.0.load();// 加载GDT到GDTR寄存器，lgdt指令，在实验报告有写，开始前预习了
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

    // 统计IST总大小
    let mut size = 0;

    for &s in IST_SIZES.iter() {
        size += s;
    }

    let (size, unit) = crate::humanized_size(size as u64);
    info!("Kernel IST Size  : {:>7.*} {}", 3, size, unit);

    info!("GDT Initialized.");
}

// ! 选择子获取接口
pub fn get_selector() -> &'static KernelSelectors {
    &GDT.1
}
