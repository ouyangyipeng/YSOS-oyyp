/*
- **核心作用**：提供物理地址与内核虚拟地址的转换能力，支撑内存管理和硬件访问。
- **关键特性**：
    1. **一次初始化**：确保物理偏移量安全设置。
    2. **高效转换**：内联函数+简单加法实现零额外开销。
    3. **强依赖约束**：强制要求初始化顺序，避免未定义行为。
- **典型调用者**：
    - **设备驱动**：访问MMIO（Memory-Mapped I/O）区域。
    - **页表管理**：构建页表时处理物理地址。
    - **物理内存分配器**：将分配的物理页框转换为可访问地址。
*/
pub const PAGE_SIZE: u64 = 4096;    // 定义内存分页的基本单位, 标准x86_64架构的4KB页大小
pub const FRAME_SIZE: u64 = PAGE_SIZE;  // 体现页帧（Frame）和页（Page）的1:1映射

// 线程安全的一次性初始化容器，确保物理偏移量仅被设置一次
pub static PHYSICAL_OFFSET: spin::Once<u64> = spin::Once::new();

pub fn init(boot_info: &'static boot::BootInfo) {
    /*
    通过引导信息boot::BootInfo获取physical_memory_offset，该值由Bootloader在启动时计算
    必须最先调用：其他模块调用physical_to_virtual前需确保init已完成。
    */
    PHYSICAL_OFFSET.call_once(|| boot_info.physical_memory_offset); // call_once原子性地设置偏移量，避免竞态条件

    info!("Physical Offset  : {:#x}", PHYSICAL_OFFSET.get().unwrap());  // 记录日志确认初始化完成
}

/// Convert a virtual address to a physical address.
#[inline(always)]
pub fn physical_to_virtual(addr: u64) -> u64 {
    /*
    - **#[inline(always)]**：强制内联优化，消除函数调用开销（高频操作）。
    - **偏移加法**：假设内核采用**直接映射（Identity Mapping Offset）**，即物理地址`PA`对应的虚拟地址`VA = PA + PHYSICAL_OFFSET`。
    - **错误处理**：若未初始化则panic，确保地址转换的可靠性。
    */
    addr + PHYSICAL_OFFSET
        .get()
        .expect("PHYSICAL_OFFSET not initialized")
}
