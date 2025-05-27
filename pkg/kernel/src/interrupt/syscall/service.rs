use core::alloc::Layout;
use crate::interrupt::clock::current_datetime;
use crate::proc;
use crate::proc::*;
use crate::utils;
use crate::utils::*;

use super::SyscallArgs;

pub fn spawn_process(args: &SyscallArgs) -> usize {
    // FIXME: get app name by args
    //       - core::str::from_utf8_unchecked
    //       - core::slice::from_raw_parts
    let name = unsafe {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(args.arg0 as *const u8, args.arg1))
    };
    // FIXME: spawn the process by name
    let ret = proc::spawn(name);
    // FIXME: handle spawn error, return 0 if failed
    // FIXME: return pid as usize
    match ret {
        Some(pid) => {
            return pid.0 as usize;
        }
        None => {
            return 0;
        }
    }
}

pub fn sys_write(args: &SyscallArgs) -> usize {
    // FIXME: get buffer and fd by args
    //       - core::slice::from_raw_parts
    let buf = unsafe {
        core::slice::from_raw_parts(args.arg1 as *const u8, args.arg2)
    };
    // FIXME: call proc::write -> isize
    let ret = proc::write(args.arg0 as u8, buf) as usize;
    // FIXME: return the result as usize
    ret
}

pub fn sys_read(args: &SyscallArgs) -> usize {
    // FIXME: just like sys_write
    let buf = unsafe {
        core::slice::from_raw_parts_mut(args.arg1 as *mut u8, args.arg2)
    };
    let ret = proc::read(args.arg0 as u8, buf) as usize;
    ret
}

pub fn sys_gettime() -> usize {
    let time = current_datetime().and_utc().timestamp_nanos_opt().unwrap_or(0);
    // let ret = utils::time_to_unix(&time);
    time as usize
}

pub fn sys_getpid() -> usize {
    proc::processor::get_pid().0 as usize
}

pub fn exit_process(args: &SyscallArgs, context: &mut ProcessContext) {
    // FIXME: exit process with retcode
    proc::exit(args.arg0 as isize, context);
}

pub fn sys_waitpid(args: &SyscallArgs, context: &mut ProcessContext) {
    let pid = ProcessId(args.arg0 as u16);
    proc::wait_process(pid, context);
    // super::super::super::wait(pid);
}

pub fn list_process() {
    // FIXME: list all processes
    proc::print_process_list();
}

pub fn sys_allocate(args: &SyscallArgs) -> usize {
    let layout = unsafe { (args.arg0 as *const Layout).as_ref().unwrap() };

    if layout.size() == 0 {
        return 0;
    }

    let ret = crate::memory::user::USER_ALLOCATOR
        .lock()
        .allocate_first_fit(*layout);

    match ret {
        Ok(ptr) => ptr.as_ptr() as usize,
        Err(_) => 0,
    }
}

pub fn sys_deallocate(args: &SyscallArgs) {
    let layout = unsafe { (args.arg1 as *const Layout).as_ref().unwrap() };

    if args.arg0 == 0 || layout.size() == 0 {
        return;
    }

    let ptr = args.arg0 as *mut u8;

    unsafe {
        crate::memory::user::USER_ALLOCATOR
            .lock()
            .deallocate(core::ptr::NonNull::new_unchecked(ptr), *layout);
    }
}

pub fn sys_fork(context: &mut ProcessContext) {
    // let ret = proc::fork(context);
    // context.set_rax(ret as usize);
    proc::fork(context);
}