use core::fmt;
use alloc::format;
use syscall_def::Syscall;
use chrono::{DateTime, FixedOffset, NaiveDateTime};

// fmt


#[inline(always)]
pub fn sys_write(fd: u8, buf: &[u8]) -> Option<usize> {
    let ret = syscall!(
        Syscall::Write,
        fd as u64,
        buf.as_ptr() as u64,
        buf.len() as u64
    ) as isize;
    if ret.is_negative() {
        None
    } else {
        Some(ret as usize)
    }
}

#[inline(always)]
pub fn sys_read(fd: u8, buf: &mut [u8]) -> Option<usize> {
    let ret = syscall!(
        Syscall::Read,
        fd as u64,
        buf.as_ptr() as u64,
        buf.len() as u64
    ) as isize;
    if ret.is_negative() {
        None
    } else {
        Some(ret as usize)
    }
}

#[inline(always)]
pub fn sys_time() -> NaiveDateTime {
    let time = syscall!(Syscall::GetTime) as i64;
    const BILLION: i64 = 1_000_000_000;
    NaiveDateTime::from_timestamp(time / BILLION, (time % BILLION) as u32)
}
#[inline(always)]
pub fn sys_time_beijing() -> DateTime<FixedOffset> {
    let time = syscall!(Syscall::GetTime) as i64;
    const BILLION: i64 = 1_000_000_000;
    let utc_dt = NaiveDateTime::from_timestamp(time / BILLION, (time % BILLION) as u32);
    let beijing_offset = FixedOffset::east_opt(8 * 3600).unwrap(); // 东八区偏移量
    DateTime::from_utc(utc_dt, beijing_offset)
}

#[inline(always)]
pub fn sys_wait_pid(pid: u16) -> isize {
    // FIXME: try to get the return value for process
    //        loop until the process is finished
    let ret = syscall!(Syscall::WaitPid, pid as u64) as isize;
    let s =format!("Process {} exited with code {}", pid, ret);
    sys_write(1, s.as_bytes());
    ret
}

#[inline(always)]
pub fn sys_list_app() {
    syscall!(Syscall::ListApp);
}

#[inline(always)]
pub fn sys_stat() {
    syscall!(Syscall::Stat);
}

#[inline(always)]
pub fn sys_allocate(layout: &core::alloc::Layout) -> *mut u8 {
    syscall!(Syscall::Allocate, layout as *const _) as *mut u8
}

#[inline(always)]
pub fn sys_deallocate(ptr: *mut u8, layout: &core::alloc::Layout) -> usize {
    syscall!(Syscall::Deallocate, ptr, layout as *const _)
}

#[inline(always)]
pub fn sys_spawn(path: &str) -> u16 {
    syscall!(Syscall::Spawn, path.as_ptr() as u64, path.len() as u64) as u16
}

#[inline(always)]
pub fn sys_get_pid() -> u16 {
    syscall!(Syscall::GetPid) as u16
}

#[inline(always)]
pub fn sys_exit(code: isize) -> ! {
    let s = format!("Process exited with code {}", code);
    sys_write(1, s.as_bytes());
    syscall!(Syscall::Exit, code as u64);
    unreachable!("This process should be terminated by now.")
}

#[inline(always)]
pub fn sys_get_time() -> usize {
    syscall!(Syscall::GetTime) as usize
}

#[inline(always)]
pub fn sys_fork() -> u16 {
    syscall!(Syscall::Fork) as u16
}