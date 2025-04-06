use crossbeam_queue::ArrayQueue;
use lazy_static::lazy_static;
use alloc::string::String;
use spin::Mutex;
use alloc::vec::Vec;

const BUFFER_SIZE: usize = 128;

type Key = u8;

lazy_static! {
    static ref INPUT_BUF: ArrayQueue<Key> = ArrayQueue::new(128);
}

lazy_static! {
    static ref UTF8_BUF: Mutex<Vec<u8>> = Mutex::new(Vec::with_capacity(4));
}

#[inline]
pub fn push_key(key: Key) {
    if INPUT_BUF.push(key).is_err() {
        warn!("Input buffer is full. Dropping key '{:?}'", key);
    }
}

#[inline]
pub fn try_pop_key() -> Option<Key> {
    INPUT_BUF.pop()
}

/// 一直循环等待输入，直到有输入为止
/// 但是这样占用cpu，所以没有数据的时候就该让出cpu
pub fn pop_key() -> Key {
    loop {
        if let Some(key) = try_pop_key() {
            return key;
        }
        // 让出cpu
        // x86_64::instructions::hlt();
        // core::hint::spin_loop(); // 这样好像也行？
    }
}

// 处理中文
pub fn get_line() -> String {
    let mut line = String::with_capacity(BUFFER_SIZE);
    let mut utf8_buf = UTF8_BUF.lock();
    
    loop {
        let key = pop_key();
        
        match key {
            b'\n' | b'\r' => {
                // println!("\n");
                return line;
            }
            0x08 | 0x7F => {
                if !line.is_empty() {
                    // 处理UTF-8字符的退格
                    let len = line.len();
                    let last_char = line.chars().last().unwrap();
                    line.pop();
                    // 根据字符宽度回退光标
                    for _ in 0..last_char.len_utf8() {
                        backspace();
                    }
                }
            }
            _ => {
                utf8_buf.push(key);
                if let Ok(s) = core::str::from_utf8(&utf8_buf) {
                    if let Some(c) = s.chars().next() {
                        line.push(c);
                        print!("{}", c);
                        utf8_buf.clear();
                    }
                } else {
                    // 需要更多字节继续等待
                    continue;
                }
            }
        }
    }
}

fn backspace() {
    print!("\x08 \x08"); // 单个字节回退
}


// /// 阻塞读取一行，匹配回车和退格，其他就正常输出到屏幕
// /// 
// pub fn get_line() -> String {
//     let mut line = String::with_capacity(BUFFER_SIZE);
//     loop {
//         let key = pop_key();
        
//         match key{
//             b'\n' | b'\r' => {
//                 // line.push('\n');
//                 println!("\n");
//                 return line;
//             }
//             0x08 | 0x7F => { // 退格键
//                 if !line.is_empty() {
//                     line.pop();
//                     backspace();
//                 }
//             }
//             _ => {
//                 let c = key as char;
//                 line.push(c);
//                 print!("{}", c);
//             }
//         }
//     }
// }

// /// 退格显示
// fn backspace() {
//     // 退格+空格覆盖+退格
//     println!("\x08 \x08"); 
// }