#![allow(dead_code, unused_imports)]
#![feature(alloc_error_handler)]
#![cfg_attr(not(test), no_std)]

#[macro_use]
pub mod macros;

#[macro_use]
extern crate syscall_def;

#[macro_use]
pub mod io;
pub mod allocator1;
pub mod allocator;
pub mod sync;
pub extern crate alloc;

mod syscall;

use core::fmt::*;

pub use alloc::*;
pub use chrono::*;
pub use io::*;
pub use sync::*;
pub use syscall::*;

pub fn init() {
    #[cfg(feature = "brk_alloc")]
    crate::allocator::init();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => ($crate::_err(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! errln {
    () => ($crate::err!("\n"));
    ($($arg:tt)*) => ($crate::err!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    stdout().write(format!("{}", args).as_str());
}

#[doc(hidden)]
pub fn _err(args: Arguments) {
    stderr().write(format!("{}", args).as_str());
}

// #![cfg_attr(not(test), no_std)]
// #![allow(dead_code, unused_imports)]
// #![feature(alloc_error_handler)]

// #[macro_use]
// pub mod macros;

// #[macro_use]
// extern crate syscall_def;

// #[macro_use]
// pub mod io;
// pub mod allocator;
// pub mod sync;
// pub extern crate alloc;

// mod syscall;

// use core::fmt::*;

// pub use alloc::*;
// pub use io::*;
// pub use syscall::*;

// #[macro_export]
// macro_rules! print {
//     ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
// }

// #[macro_export]
// macro_rules! println {
//     () => ($crate::print!("\n"));
//     ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
// }

// #[macro_export]
// macro_rules! err {
//     ($($arg:tt)*) => ($crate::_err(format_args!($($arg)*)));
// }

// #[macro_export]
// macro_rules! errln {
//     () => ($crate::err!("\n"));
//     ($($arg:tt)*) => ($crate::err!("{}\n", format_args!($($arg)*)));
// }

// #[doc(hidden)]
// pub fn _print(args: Arguments) {
//     stdout().write(format!("{}", args).as_str());
// }

// #[doc(hidden)]
// pub fn _err(args: Arguments) {
//     stderr().write(format!("{}", args).as_str());
// }
