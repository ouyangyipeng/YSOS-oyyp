#![no_std]
#![no_main]
#![feature(alloc_error_handler)]  // 如果需要处理内存分配错误

extern crate alloc;
use alloc::string::String;
use alloc::format;

#[macro_use]
extern crate log;

use ysos::*;

// use core::arch::asm;
use ysos_kernel as ysos;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);
    info!("Kernel initialized.");
    info!("Hello World from YatSenOS v2!");

    // FIXME: update lib.rs to pass following tests

    // 1. run some (about 5) "test", show these threads are running concurrently

    // 2. run "stack", create a huge stack, handle page fault properly

    let mut test_num = 0;

    // run 5 test
    // for _ in 0..50 {
    //     ysos::new_test_thread(format!("{}", test_num).as_str());
    //     test_num += 1;
    // }

    // run 1 stack
    // ysos::new_stack_test_thread();

    loop {
        print!("[>] ");
        let line = input::get_line();
        match line.trim() {
            "exit" => break,
            "ps" => {
                ysos::proc::print_process_list();
            }
            "stack" => {
                ysos::new_stack_test_thread();
            }
            "test" => {
                ysos::new_test_thread(format!("{}", test_num).as_str());
                test_num += 1;
            }
            _ => println!("[=] {}", line),
        }
    }

    // loop {
    //     let counter = interrupt::clock::read_counter() / 10000;
        
    //     // print!("> ");
    //     // print!(
    //     //     "\x1b[34m░▒▓\
    //     //     \x1b[44m\x1b[37m /work/OYOS\
    //     //     \x1b[43m\x1b[30m main !5 \
    //     //     \x1b[33m\
    //     //     \x1b[30m\
    //     //     \x1b[40m\x1b[31m ✔ │ root@Owen  \
    //     //     \x1b[47m\x1b[30m{} 
    //     //     \x1b[37m▓▒░\x1b[0m ",
    //     //     format_time(counter)
    //     // );
    //     format_prompt(counter);
    //     println!();
    //     print!("╰─ ");
    //     let input1 = input::get_line();

    //     let input = input1.trim();
    //     if input.is_empty() {
    //         continue;
    //     }

    //     match input {
    //         "exit" => {
    //             println!("Exiting...");
    //             break
    //         },
    //         "help" => {
    //             println!("Available commands:");
    //             println!("  exit - Exit the kernel");
    //             println!("  help - Show this help message");
    //             println!("  clock - Show the current clock counter value");
    //             println!("  echo <message> - Print the message to the console");
    //         }
    //         "clock" => {
    //             println!("The current clock counter value is {}", interrupt::clock::read_counter());
    //         }
    //         "echo" => {
    //             println!("Usage: echo <message>");
    //             println!("Prints the message to the console.");
    //         }
    //         _ if input.starts_with("echo ") => {
    //             let message = &input[5..];
    //             println!("{}", message);
    //         }
    //         _ => {
    //             // println!("You said: {}", input);
    //             // println!("The counter value is {}", interrupt::clock::read_counter());
    //             println!("Unknown command: {}", input);
    //             println!("Type 'help' for a list of available commands.");
    //         }
    //     }
    // }
    ysos::shutdown();
}

fn format_time(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

fn format_prompt(counter: u64) {
    // 获取终端宽度，需要以后实现终端尺寸查询
    // 这里还有很多内容比如文件系统什么的，都后面再实现，先做了个样子
    let term_width: i32 = 80; // 默认值80
    
    // 左侧部分
    let left = format!(
        "╭─\x1b[34m░▒▓\x1b[44m\x1b[37m /work/OYOS\x1b[43m\x1b[30m main !5 \x1b[33m\x1b[40m"
    );
    
    // 右侧部分
    let right = format!(
        "\x1b[30m\x1b[40m\x1b[31m 😄✅ │ root@Owen \x1b[47m\x1b[30m{} \x1b[37m\x1b[40m▓▒░\x1b[0m─╮",
        format_time(counter)
    );
    // 🤬❌
    // 🤔⚠️
    
    // 计算填充宽度
    let left_len: i32 = 22; // 实际显示字符数
    let right_len: i32 = 25; // 实际显示字符数
    let fill_width: i32 = term_width.saturating_sub(left_len + right_len);
    
    print!(
        "{}\x1b[0m{:─<width$}{}\x1b[0m",
        left, "", right, width = fill_width as usize
    );
}