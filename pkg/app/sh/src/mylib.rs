use alloc::string::String;
extern crate lib;
use lib::*;
use chrono::Timelike;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const BLINK: &str = "\x1b[5m";
const DIM: &str = "\x1b[2m";  // 暗淡效果，用于制造阴影
const R1: &str = "\x1b[91m"; // 亮红
const R2: &str = "\x1b[93m"; // 亮黄
const R3: &str = "\x1b[92m"; // 亮绿
const R4: &str = "\x1b[96m"; // 亮青
const R5: &str = "\x1b[94m"; // 亮蓝
const R6: &str = "\x1b[95m"; // 亮洋红
const RAINBOW: [&str; 6] = [R1, R2, R3, R4, R5, R6]; // 颜色数组

pub fn format_time(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

pub fn format_prompt(counter: u64) {
    // 获取终端宽度，需要以后实现终端尺寸查询
    // 这里还有很多内容比如文件系统什么的，都后面再实现，先做了个样子
    let term_width: i32 = 80; // 默认值80
    
    // 左侧部分
    let left = format!(
        "╭─\x1b[34m░▒▓\x1b[44m\x1b[37m /work/OYOS\x1b[43m\x1b[30m main !5 \x1b[33m\x1b[40m"
    );
    let beijing_time = sys_time_beijing();
    
    // 右侧部分
    let right = format!(
        "\x1b[30m\x1b[40m\x1b[31m 😄✅ │ root@Owen \x1b[47m\x1b[30m{:02}:{:02}:{:02} \x1b[37m\x1b[40m▓▒░\x1b[0m─╮",
        beijing_time.hour(), beijing_time.minute(), beijing_time.second()
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

pub fn help() {
    println!("Available commands:");
    println!("  ps - Show process status");
    println!("  la - List all applications");
    println!("  clear - Clear the screen");
    println!("  exit - Exit the kernel");
    println!("  help - Show this help message");
    println!("  clock - Show the current clock counter value");
    println!("  echo <message> - Print the message to the console");
}

pub fn run(path: &str) {
    let name: vec::Vec<&str> = path.rsplit('/').collect();
    let pid = sys_spawn(path);
    if pid == 0 {
        println!("{BOLD}{R1}⚠ Failed to run app: {}{RESET}", name[0]);
    } else {
        sys_stat();
        println!("{BOLD}{R3}✓ {} exited with {}{RESET}", name[0], sys_wait_pid(pid));
        // loop {
        //     let ret = sys_wait_pid(pid);
        //     if ret == 2333 {
        //         println!("{BOLD}{R1}⚠ {} is still running...{RESET}", name[0]);
        //     } else {
        //         println!("{BOLD}{R1}⚠ {} exited with {}{RESET}, yeah", name[0], ret);
        //         break;
        //     }
        // }
        // loop {
        //     let ret = sys_wait_pid(pid);
        //     if ret == 0 {
        //         println!("{BOLD}{R1}⚠ {} exited with {}{RESET}, yeah", name[0], ret);
        //         break;
        //     }
        // }
    }
}

pub fn echo(message: &str) {
    if message.is_empty() {
        println!("Usage: echo <message>");
        println!("Prints the message to the console.");
    } else{
        println!("{}{}", BOLD, message);
    }
}

pub fn output_banner() {
    let banner =[
        "╭───────────────────────────────────────────────────────────────╮",
        "│  ██████╗ ██╗   ██╗ ██████╗ ███████╗██╗   ██╗███╗   ██╗ ██████╗│",
        "│ ██╔═══██╗╚██╗ ██╔╝██╔═══██╗██╔════╝╚██╗ ██╔╝████╗  ██║██╔════╝│",
        "│ ██║   ██║ ╚████╔╝ ██║   ██║███████╗ ╚████╔╝ ██╔██╗ ██║██║     │",
        "│ ██║   ██║  ╚██╔╝  ██║   ██║╚════██║  ╚██╔╝  ██║╚██╗██║██║     │",
        "│ ╚██████╔╝   ██║   ╚██████╔╝███████║   ██║   ██║ ╚████║╚██████╗│",
        "│  ╚═════╝    ╚═╝    ╚═════╝ ╚══════╝   ╚═╝   ╚═╝  ╚═══╝ ╚═════╝│",
        "│                                                               │",
        "│                 OYOSync - Owen's Yet Another OS               │",
        "│                                                               │",
        "╰───────────────────────────────────────────────────────────────╯"
    ];
    for (i, line) in banner.iter().enumerate() {

        print!("   {DIM}");
        // for ch in line.chars() {
        //     if ch != ' ' {
        //         print!("█");
        //     } else {
        //         print!(" ");
        //     }
        // }
        println!("{RESET}");

        print!("\x1B[1A\x1B[2C"); 
        let color = RAINBOW[i % RAINBOW.len()];
        println!("{BOLD}{color}{}{RESET}", line);
    }
    let info = "学号: 23336188   姓名: 欧阳易芃";

    print!("   {DIM}");
    for ch in info.chars() {
        if ch != ' ' {
            print!("▓");
        } else {
            print!(" ");
        }
    }
    println!("{RESET}");
    print!("\x1B[1A\x1B[2C"); 
    for (i, ch) in info.chars().enumerate() {
        let color = RAINBOW[i % RAINBOW.len()];
        print!("{BOLD}{color}{}", ch);
    }
    println!("{RESET}"); 
    
    let welcome = "✨ 欢迎使用 YatSenOS 终端 ✨";

    print!("   {DIM}");
    for ch in welcome.chars() {
        if ch != ' ' {
            print!("▒");
        } else {
            print!(" ");
        }
    }
    println!("{RESET}");
    

    print!("\x1B[1A\x1B[2C"); 
    println!("{BOLD}{BLINK}{R4}{}{RESET}", welcome);
    

    let help_text = "输入 'help' 查看可用指令";
    print!("   ");
    for (i, ch) in help_text.chars().enumerate() {
        let color = RAINBOW[i % RAINBOW.len()];
        print!("{BOLD}{color}{}", ch);
    }
    println!("{RESET}\n");
}