use crate::drivers::serial::get_serial;
use core::fmt::*;
use x86_64::instructions::interrupts;
use crate::drivers::serial::SERIAL;
/*
1. **线程安全初始化**：
    - 提供`once_mutex!`创建延迟初始化的互斥锁
    - 自动生成带错误检查的访问接口（`guard_access_fn!`）
2. **调试输出控制**：
    - 实现`print!`/`println!`宏，支持格式化输出
    - 保证输出操作的原子性和可靠性
3. **系统健壮性保障**：
    - 定制化panic处理流程
    - 关键资源（如串口）的应急解锁机制
4. **代码生成优化**：
    - 使用`paste`库简化标识符拼接
    - 通过属性控制内联行为
*/


// ! 同步原语宏
/// Use spin mutex to control variable access
#[macro_export]
macro_rules! guard_access_fn {
    /* 为互斥锁生成访问接口 */
    ($(#[$meta:meta])* $v:vis $fn:ident ($mutex:path:$ty:ty)) => {
        paste::item! {

            $(#[$meta])*
            #[inline(never)]    // 避免锁操作被内联优化
            #[allow(non_snake_case, dead_code)]
            $v fn $fn<'a>() -> Option<spin::MutexGuard<'a, $ty>> {
                /* 尝试获取锁，返回Option */
                $mutex.get().and_then(spin::Mutex::try_lock)
            }

            $(#[$meta])*
            #[inline(never)]
            #[allow(non_snake_case, dead_code)]
            $v fn [< $fn _for_sure >]<'a>() -> spin::MutexGuard<'a, $ty> {
                /* 必须获取锁，失败则panic */
                $mutex.get().and_then(spin::Mutex::try_lock).expect(
                    stringify!($mutex has not been initialized or lockable)
                )   // 自动生成带错误信息的panic消息
            }
        }
    };
}

#[macro_export]
macro_rules! once_mutex {
    /* 创建线程安全的延迟初始化互斥锁 */
    ($i:vis $v:ident: $t:ty) => {
        $i static $v: spin::Once<spin::Mutex<$t>> = spin::Once::new();
        // 使用spin::Once保证一次性初始化
        paste::item! {  // 通过paste::item!实现标识符拼接
            #[allow(non_snake_case)]
            $i fn [<init_ $v>]([<val_ $v>]: $t) {   // 自动生成带init_前缀的初始化函数
                $v.call_once(|| spin::Mutex::new([<val_ $v>]));
            }
        }
    };
}

// ! 调试输出系统
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::utils::print_internal(format_args!($($arg)*))   // 使用标准库format_args!捕获格式化参数
    );
}

#[macro_export]
macro_rules! println {
    /* 自动添加\n\r换行符（兼容Windows和Unix） */
    () => ($crate::print!("\n\r"));
    ($($arg:tt)*) => ($crate::print!("{}\n\r", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn print_internal(args: Arguments) {
    /* 通过串口设备实现输出 */
    interrupts::without_interrupts(|| { // without_interrupts保证原子性写入
        if let Some(mut serial) = get_serial() {
            serial.write_fmt(args).unwrap();    // 通过write_fmt实现格式化输出
        }
    });
}

// ! 异常处理
#[allow(dead_code)]
#[cfg_attr(target_os = "none", panic_handler)]  // target_os: 仅在没有OS的环境生效; panic_handler指定为全局panic处理器
fn panic(info: &core::panic::PanicInfo) -> ! {
    // force unlock serial for panic output
    unsafe { SERIAL.get().unwrap().force_unlock() };    // 强制解锁串口（避免死锁状态无法输出）

    error!("ERROR: panic!\n\n{:#?}", info); // 通过串口输出详细的panic信息
    loop {} // 进入死循环（停止系统运行）
}
