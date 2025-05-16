use crate::*;
use alloc::string::{String, ToString};
use alloc::vec;
use crossbeam_queue::ArrayQueue;
use lazy_static::lazy_static;
use spin::Mutex;
use alloc::vec::Vec;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;
// lazy_static! {
// static ref UTF8_BUF_IO: Mutex<Vec<u8>> = Mutex::new(Vec::with_capacity(4))
// }

impl Stdin {
    fn new() -> Self {
        Self
    }

    pub fn read_line(&self) -> String {
        // FIXME: allocate string
        // FIXME: read from input buffer
        //       - maybe char by char?
        // FIXME: handle backspace / enter...
        // FIXME: return string
        // 参考之前input里面写的
        let mut line = String::new();
        let mut input_buffer = Vec::new(); // 存储从系统读取的字节
        let mut utf8_partial = Vec::new(); // 累积多字节UTF-8字符的临时缓冲区

        loop {
            // 当需要更多数据时读取输入
            if input_buffer.is_empty() {
                let mut temp_buf = [0u8; 256];
                if let Some(n) = sys_read(0, &mut temp_buf) {
                    input_buffer.extend_from_slice(&temp_buf[..n]);
                } else {
                    continue; // 读取失败，重试
                }
            }

            // 处理缓冲区的每个字节
            while !input_buffer.is_empty() {
                let byte = input_buffer.remove(0); // 取出第一个字节处理

                match byte {
                    b'\n' | b'\r' => { // 回车或换行，结束输入
                        sys_write(1, b"\n");
                        return line;
                    }
                    0x08 | 0x7F => { // 处理退格
                        if !line.is_empty() {
                            let c = line.pop().unwrap();
                            let backspace_seq = b"\x08 \x08"; // 退格、空格、退格
                            // 根据字符的UTF-8长度回退光标
                            for _ in 0..c.len_utf8() {
                                sys_write(1, backspace_seq);
                            }
                        }
                    }
                    _ => { // 普通字符处理
                        utf8_partial.push(byte);
                        match core::str::from_utf8(&utf8_partial) {
                            Ok(s) => {
                                if let Some(c) = s.chars().next() {
                                    // 成功解析字符，添加到行并显示
                                    line.push(c);
                                    sys_write(1, c.to_string().as_bytes());
                                    utf8_partial.clear(); // 清空临时缓冲区
                                }
                            }
                            Err(e) => {
                                if e.error_len().is_some() {
                                    // 无效UTF-8序列，丢弃已累积的字节
                                    utf8_partial.clear();
                                }
                                // 否则继续累积更多字节
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Stdout {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(1, s.as_bytes());
    }
}

impl Stderr {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(2, s.as_bytes());
    }
}

pub fn stdin() -> Stdin {
    Stdin::new()
}

pub fn stdout() -> Stdout {
    Stdout::new()
}

pub fn stderr() -> Stderr {
    Stderr::new()
}
