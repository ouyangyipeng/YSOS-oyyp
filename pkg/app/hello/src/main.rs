#![no_std]
#![no_main]

use lib::{boxed::Box, *};

extern crate lib;

fn main() -> isize {
    println!("Hello, world!!!");
    
    // unsafe{let out = *(0xFFFFFFFF00000000);}    
    // println!("{}", out);
    // let a = Box::new(1919810);
    // let pid = sys_fork();
    // println!("Box{}{}", a, pid);

    233
}

entry!(main);
