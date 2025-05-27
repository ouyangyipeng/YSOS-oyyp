use core::{
    hint::spin_loop,               // 用于自旋锁优化的CPU提示指令
    sync::atomic::{AtomicBool, Ordering}, // 原子布尔类型和内存顺序
};

use crate::*;                      // 引入其他库模块

pub struct SpinLock {
    bolt: AtomicBool,               // 原子布尔值表示锁状态（false=未锁定，true=锁定）
}

impl SpinLock {
    pub const fn new() -> Self {    // 编译期常量构造函数
        Self {
            bolt: AtomicBool::new(false), // 初始化锁为未锁定状态
        }
    }

    pub fn acquire(&self) {         // 获取锁的方法（文档要求填充实现）
        // FIXME: 使用原子操作检查锁状态，若已被占用则循环等待
        // 实验文档提示：使用 compare_exchange 实现原子状态切换
        // 示例实现：
        // while self.bolt.compare_exchange_weak(
        //     false,                  // 预期当前值为未锁定
        //     true,                   // 尝试设置为锁定
        //     Ordering::Acquire,       // 内存顺序：获取语义（后续操作不会重排到此之前）
        //     Ordering::Relaxed       // 失败时的内存顺序
        // ).is_err() {
        //     spin_loop();            // 忙等待时优化CPU功耗（x86的pause指令）
        // }
    }

    pub fn release(&self) {         // 释放锁的方法（文档要求填充实现）
        // FIXME: 原子地将锁状态重置为未锁定
        // 正确实现：
        // self.bolt.store(false, Ordering::Release); // 释放语义（前序操作不会重排到此之后）
    }
}

unsafe impl Sync for SpinLock {}    // 标记该类型可安全跨线程共享（思考题5关键点）

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Semaphore {  // 信号量
    /* FIXME: record the sem key */ // 根据文档，需保存内核信号量的唯一标识符
    // key: u32,                       // 补充字段：与内核信号量关联的键值
}

impl Semaphore {
    pub const fn new(key: u32) -> Self { // 用户态信号量构造函数
        // Semaphore { key }           // 仅记录key，实际资源在内核管理
    }

    #[inline(always)]
    pub fn init(&self, value: usize) -> bool { // 初始化信号量（对应文档中的sem系统调用）
        sys_new_sem(self.key, value) // 系统调用创建信号量（op=0）
    }

    /* FIXME: other functions with syscall... */
    // 根据文档补充：
    // #[inline(always)]
    // pub fn wait(&self) -> isize {   // P操作（op=3）
    //     syscall!(Syscall::Sem, 3, self.key as usize, 0)
    // }

    // #[inline(always)]
    // pub fn signal(&self) -> isize { // V操作（op=2）
    //     syscall!(Syscall::Sem, 2, self.key as usize, 0)
    // }
}

unsafe impl Sync for Semaphore {}   // 同上，标记线程安全

#[macro_export]
macro_rules! semaphore_array {      // 辅助宏（用于哲学家问题的5个信号量）
    [$($x:expr),+ $(,)?] => {
        [ $( $crate::Semaphore::new($x), )* ] // 生成Semaphore实例数组
    }
}

/*
​​自旋锁的原子性​​
    compare_exchange_weak 实现无锁状态检查，确保多核环境下不会同时获取锁
    Ordering::Acquire/Release 确保临界区内存访问顺序（文档中的原子指令章节）
​​信号量用户-内核交互​​
    用户态仅保存 key，实际资源由内核的 SemaphoreSet 管理（文档中的信号量系统调用设计）
    sys_new_sem 对应 op=0 的系统调用（文档中 sys_sem 的参数约定）
​​Sync标记的必要性​​
    自旋锁需跨线程共享（如文档中的多线程计数器测试），unsafe impl Sync 告诉编译器该类型的线程安全性
*/