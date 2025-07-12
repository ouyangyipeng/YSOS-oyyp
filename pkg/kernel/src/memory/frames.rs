use alloc::boxed::Box;
use boot::{MemoryMap, MemoryType};
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB};    // 页帧类型和分配器trait
use x86_64::PhysAddr;   // 物理地址类型
use alloc::vec::Vec;

// ! 同步原语宏（自定义）:实现线程安全的单例访问模式
once_mutex!(pub FRAME_ALLOCATOR: BootInfoFrameAllocator);   // 创建延迟初始化的互斥锁

guard_access_fn! {
    pub get_frame_alloc(FRAME_ALLOCATOR: BootInfoFrameAllocator)    // 生成安全的访问接口，强制调用者持有锁
}
const RS_ALIGN_4KIB: u64 = 12;

// ! 页帧迭代器类型
type BootInfoFrameIter = Box<dyn Iterator<Item = PhysFrame> + Send>;

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    size: usize,
    used: usize,
    frames: BootInfoFrameIter,
    // recycled: Vec<u32>,
    recycled: Vec<PhysFrame>,
}

// ! 主结构体
impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &MemoryMap, size: usize) -> Self {   // 必须确保标记为USABLE的内存区域确实未被使用，过程：从引导信息的内存映射（MemoryMap）创建迭代器，记录总可用页帧数（由调用者计算提供）
        BootInfoFrameAllocator {
            size,       // 总可用页帧数
            frames: create_frame_iter(memory_map),  // 页帧迭代器:动态分发的迭代器（Box<dyn Iterator>），支持遍历所有可用物理页帧
            used: 0,    // 已分配页帧计数
            recycled: Vec::new(),   // 回收页帧列表
        }
    }

    pub fn frames_used(&self) -> usize {
        self.used
    }

    pub fn frames_total(&self) -> usize {
        self.size
    }

    pub fn recycled_count(&self) -> usize {
        self.recycled.len() as usize
    }
    pub fn frames_recycled(&self) -> usize {
        self.recycled.len()
    }
}

// ! 实现分配器接口
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        /* 每次调用从迭代器获取下一个可用页帧，递增used计数器（即使返回None也计数，可能存在统计偏差） */
        // self.used += 1;
        // self.frames.next()
        if let Some(frame) = self.recycled.pop() {
            // Some(u32_to_phys_frame(frame))
            Some(frame)
        } else {
            self.used += 1;
            self.frames.next()
        }
    }
}

impl FrameDeallocator<Size4KiB> for BootInfoFrameAllocator {
    unsafe fn deallocate_frame(&mut self, _frame: PhysFrame) {
        // TODO: deallocate frame (not for lab 2)
        // let key = phys_frame_to_u32(_frame);
        let key = _frame;
        self.recycled.push(key);
    }
}

#[inline(always)]
fn phys_frame_to_u32(frame: PhysFrame) -> u32 {
    let key = frame.start_address().as_u64() >> RS_ALIGN_4KIB;

    assert!(key <= u32::MAX as u64);

    key as u32
}

#[inline(always)]
fn u32_to_phys_frame(key: u32) -> PhysFrame {
    PhysFrame::containing_address(PhysAddr::new((key as u64) << RS_ALIGN_4KIB))
}

// ! 工具函数
fn create_frame_iter(memory_map: &MemoryMap) -> BootInfoFrameIter {
    /*
    内存布局示例：
    - 物理区域: 0x1000-0x5000 (16KB)
    - 生成页帧: [0x1000, 0x2000, 0x3000, 0x4000]
    */
    let iter = memory_map
        .clone()
        .into_iter()
        // get usable regions from memory map
        .filter(|r| r.ty == MemoryType::CONVENTIONAL)   // 过滤出可用内存区域，仅保留CONVENTIONAL（常规可用）内存区域
        // align to page boundary
        .flat_map(|r| (0..r.page_count).map(move |v| (v * 4096 + r.phys_start)))    // 过滤可用区域的页帧地址，将每个内存区域按4KB页大小拆分为连续页帧
        // create `PhysFrame` types from the start addresses
        .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)));    // 转换为页帧类型

    Box::new(iter)
}

