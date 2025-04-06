#![no_std]
#![no_main]

#[macro_use]
extern crate log;

use ysos::*;

use core::arch::asm;
use ysos_kernel as ysos;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);
    info!("Kernel initialized.");
    loop {
        info!("Hello World from YatSenOS v2!");
        print!("> ");
        let input = input::get_line();

        match input.trim() {
            "exit" => {
                println!("Exiting...");
                break
            },
            _ => {
                println!("You said: {}", input);
                println!("The counter value is {}", interrupt::clock::read_counter());
            }
        }
    }
    ysos::shutdown();
}
