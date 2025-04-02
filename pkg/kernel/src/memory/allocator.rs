use linked_list_allocator::LockedHeap;
use x86_64::VirtAddr;
use core::ptr::addr_of_mut;

pub const HEAP_SIZE: usize = 8 * 1024 * 1024; // 8 MiB

/// Use linked_list_allocator for kernel heap
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty(); // 定义内核堆的静态内存大小
// LockedHeap：基于链表的线程安全分配器，通过自旋锁（spin::Mutex）保证并发安全。
// empty()表示初始时无可用堆内存，需后续通过init()提供内存区域。
// #[global_allocator]：标记为Rust的全局分配器，接管所有Box/Vec/String等动态内存分配。

// ! 堆初始化
pub fn init() {
    // 在BSS段预留静态数组作为堆内存
    // static buffer for kernel heap
    // will be allocated on the bss section when the kernel is load
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];  // 在内核BSS段创建8MB的静态数组，所有元素初始化为0。

    // 计算堆地址范围；heap_start和heap_end用于日志显示堆的内存范围
    let heap_start = VirtAddr::from_ptr(addr_of_mut!(HEAP));    // 将裸指针转换为类型安全的虚拟地址对象；使用addr_of_mut!安全地获取可变指针（避免直接引用未初始化内存）
    let heap_end = heap_start + HEAP_SIZE as u64;

    // 初始化分配器
    unsafe {
        ALLOCATOR.lock().init(addr_of_mut!(HEAP) as *mut u8, HEAP_SIZE);    // ALLOCATOR.lock()获取自旋锁，调用init()将HEAP数组的内存区域提供给分配器
    }

    // 日志输出
    debug!(
        "Kernel Heap      : 0x{:016x}-0x{:016x}",
        heap_start.as_u64(),
        heap_end.as_u64()
    );

    let (size, unit) = crate::humanized_size(HEAP_SIZE as u64); // 使用crate::humanized_size转换8MB为"8.000 MiB"格式
    info!("Kernel Heap Size : {:>7.*} {}", 3, size, unit);

    info!("Kernel Heap Initialized.");
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    /* 
    - **触发条件**：当内存分配请求无法满足时（如堆内存耗尽）。
    - **行为**：直接触发内核panic，打印错误的内存布局（请求大小和对齐）。
    - **设计意义**：避免在内存不足时继续执行导致未定义行为。
     */
    panic!("Allocation error: {:?}", layout);
}
