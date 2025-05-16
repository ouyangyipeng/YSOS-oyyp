#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() -> isize {
    println!("Hello, world!!!");
    
    // unsafe{let out = *(0xFFFFFFFF00000000);}    
    // println!("{}", out);

    233
}

entry!(main);
