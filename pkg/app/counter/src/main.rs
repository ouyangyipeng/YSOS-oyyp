#![no_std]                     // 禁用标准库（裸机环境）
#![no_main]                    // 禁用主函数宏（自定义入口）
use lib::*;                    // 引入自定义库（含系统调用）
extern crate lib;

const THREAD_COUNT: usize = 8; // 定义并发线程数
static mut COUNTER: isize = 0; // 全局共享变量（无保护，存在竞态风险）

fn main() -> isize {
    let mut pids = [0u16; THREAD_COUNT]; // 存储子进程PID的数组

    // 创建 THREAD_COUNT 个子进程
    for i in 0..THREAD_COUNT {
        let pid = sys_fork(); // 调用 fork 系统调用
        if pid == 0 {         // 子进程分支
            do_counter_inc(); // 执行计数器累加操作
            sys_exit(0);      // 子进程退出
        } else {              // 父进程分支
            pids[i] = pid;    // 记录子进程PID
        }
    }

    // 父进程后续逻辑
    let cpid = sys_get_pid(); // 获取当前进程PID
    println!("process #{} holds threads: {:?}", cpid, &pids);
    sys_stat(); // 打印进程状态（调试用）

    // 等待所有子进程退出
    for i in 0..THREAD_COUNT {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]); // 阻塞等待指定PID的子进程
    }

    // 输出最终计数器值（因竞态问题，结果不确定）
    println!("COUNTER result: {}", unsafe { COUNTER });
    0
}

fn do_counter_inc() {
    for _ in 0..100 {
        // FIXME: protect the critical section 此处需添加锁保护（临界区）
        inc_counter(); // 无锁调用，导致竞态条件
    }
}

/// Increment the counter
///
/// this function simulate a critical section by delay
/// DO NOT MODIFY THIS FUNCTION
/// 模拟非原子操作的计数器递增（故意暴露竞态条件）
#[inline(never)] // 禁止内联优化，确保每次完整执行
#[unsafe(no_mangle)]
fn inc_counter() {
    unsafe {
        delay();          // 模拟操作延迟（增加竞态概率）
        let mut val = COUNTER; // 读取当前值
        delay();
        val += 1;         // 修改值（非原子）
        delay();
        COUNTER = val;     // 写回新值（可能覆盖其他进程的修改）
    }
}

/// 延迟函数（模拟临界区执行时间）
#[inline(never)]
#[unsafe(no_mangle)]
fn delay() {
    for _ in 0..0x100 {   // 空循环产生延迟
        core::hint::spin_loop();
    }
}

entry!(main); // 自定义入口宏（启动 main 函数）