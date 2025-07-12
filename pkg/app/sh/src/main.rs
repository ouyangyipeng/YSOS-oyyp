#![no_std]
#![no_main]
extern crate lib;
use lib::*;
// String
// use alloc::string::{String, ToString};
// Vec
use alloc::vec::Vec;
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use chrono::Timelike;

mod mylib;
use mylib::*;

fn main() -> isize {
    print!("\x1B[2J\x1B[H");// 清屏
    // 这里的颜色代码参考了nxh同学的实现，因为觉得他那个很好看（doge

    println!("\n\n");

    output_banner();

    loop {
        let counter = 54000+13*60+30; // interrupt::clock::read_counter();
        format_prompt(counter);
        println!();
        print!("╰─ ");
        let binding = stdin().read_line();
        let mut command: core::str::Split<'_, char> = binding.trim().split(' ');
        let input = command.next().unwrap();
        if input.is_empty() {
            continue;
        }

        match input {
            "exit" | "quit" | "q" => {
                println!("Exiting...");
                break
            }
            "ps" => {
                sys_stat();
            }
            "la" => {
                sys_list_app();
            }
            "clear" => {
                print!("\x1B[2J\x1B[H"); 
            }
            "help" => {
                help();
            }
            "clock" => {
                // println!("The current clock counter value is {}", format_time(counter));
                // println!(
                let beijing_time = sys_time_beijing();
                // println!("当前时间: {}", beijing_time.format("%H:%M:%S")); // 例如: 15:30:45
                println!("时分秒: {:02}:{:02}:{:02}", beijing_time.hour(), beijing_time.minute(), beijing_time.second());
            }
            "echo" => {
                let message = command.collect::<Vec<&str>>().join(" ");
                echo(message.as_str());
            }
            "run" => {
                let path = command.next().unwrap();
                run(path);
            }
            "ls" =>{
                  sys_list_dir(command.next().unwrap_or("/"));
            }
            "cat" => {
                let fd = sys_open_file(command.next().unwrap_or(""));
                let buf = &mut [0u8; 1024];
                sys_read(fd, buf);
                println!(
                    "{}",
                    core::str::from_utf8(buf).unwrap_or("Failed to read file")
                );
                sys_close_file(fd);
            }
            _ => {
                println!("Unknown command: {}", input);
                println!("Type 'help' for a list of available commands.");
            }
        }
    }
    0
}


entry!(main);
