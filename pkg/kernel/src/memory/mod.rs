pub mod address;// 地址相关的
pub mod allocator;// 分配器相关
mod frames;// 帧相关（内部私有

pub mod gdt;// GDT相关

pub use address::*;// 导出地址模块的所有公共
pub use frames::*;// 帧模块

use crate::humanized_size;

// ! 内存初始化，解析UEFI的内存映射表
pub fn init(boot_info: &'static boot::BootInfo) {
    let memory_map = &boot_info.memory_map;

    // 物理内存大小计算
    let mut mem_size = 0;
    let mut usable_mem_size = 0;

    for item in memory_map.iter() {// 遍历引导程序提供的memory_map
        mem_size += item.page_count;// 计算总物理内存页数
        if item.ty == boot::MemoryType::CONVENTIONAL {// 统计标记为CONVENTIONAL（常规可用）的内存页数
            usable_mem_size += item.page_count;
        }
    }

    // 输出内存信息
    let (size, unit) = humanized_size(mem_size * PAGE_SIZE);
    info!("Physical Memory    : {:>7.*} {}", 3, size, unit);

    let (size, unit) = humanized_size(usable_mem_size * PAGE_SIZE);
    info!("Free Usable Memory : {:>7.*} {}", 3, size, unit);

    // 初始化帧分配器
    unsafe {
        init_FRAME_ALLOCATOR(BootInfoFrameAllocator::init(// 创建基于引导信息的页帧分配器，通过init_FRAME_ALLOCATOR（来自frames模块）注册全局分配器
            memory_map,
            usable_mem_size as usize,
        ));
    }

    info!("Frame Allocator initialized.");
}
