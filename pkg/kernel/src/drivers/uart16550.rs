use core::fmt;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

/// A port-mapped UART 16550 serial interface.
pub struct SerialPort{
    base_port: u16,
}

// 注：由于有些x86 crate中的内容我没太看懂，因此本文件的部分代码实现中使用了copilot的修复功能来修复了一些错误，例如write方法之类
impl SerialPort {
    pub const fn new(port: u16) -> Self {
        Self{
            base_port: port,
        }
    }

    /// Initializes the serial port.
    pub fn init(&self) {
        // FIXME: Initialize the serial port
        let base = self.base_port;
        unsafe{
            // 禁用所有中断（中断使能寄存器）
            PortWriteOnly::new(base + 1).write(0x00 as u8);

            // 启用DLAB（除数锁存访问位，线控制寄存器）
            PortWriteOnly::new(base + 3).write(0x80 as u8);

            // 设置波特率除数（低字节和高字节）
            PortWriteOnly::new(base + 0).write(0x03 as u8); // 低字节 0x03 -> 38400 baud
            PortWriteOnly::new(base + 1).write(0x00 as u8); // 高字节 0x00

            // 配置数据格式：8位数据，无校验，1停止位（线控制寄存器）
            PortWriteOnly::new(base + 3).write(0x03 as u8);

            // 启用FIFO，清空缓冲区，14字节阈值（FIFO控制寄存器）
            PortWriteOnly::new(base + 2).write(0xC7 as u8);

            // 设置调制解调器控制寄存器：启用RTS/DSR，启用中断
            PortWriteOnly::new(base + 4).write(0x0B as u8);

            // 进入回环测试模式（调制解调器控制寄存器）
            PortWriteOnly::new(base + 4).write(0x1E as u8);

            // 发送测试字节0xAE，验证回环
            PortWriteOnly::new(base).write(0xAE as u8);

            // 检查接收数据是否正确
            let received: u8 = PortReadOnly::new(base).read();
            if received != 0xAE {
                panic!("Serial port test failed: expected 0xAE, got {:#x}", received);
            }

            // 设置正常操作模式（禁用回环，启用中断）
            PortWriteOnly::new(base + 4).write(0x0F as u8);
        }
    }

    /// Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        // FIXME: Send a byte on the serial port
        let base = self.base_port;
        // 等待发送保持寄存器为空（线状态寄存器第5位）
        unsafe{while (PortReadOnly::<u8>::new(base + 5).read() & 0x20) == 0 {}};
        // 写入数据
        unsafe { PortWriteOnly::new(base).write(data) };
    }

    /// Receives a byte on the serial port no wait.
    pub fn receive(&mut self) -> Option<u8> {
        // FIXME: Receive a byte on the serial port no wait
        let base = self.base_port;
        // 检查数据就绪位（线状态寄存器第0位）
        unsafe{
            if (PortReadOnly::<u8>::new(base + 5).read() & 0x01) != 0 {
                Some(PortReadOnly::<u8>::new(base).read())
            } else {
                None
            }
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
            // 处理换行符，添加回车
            if byte == b'\n' {
                self.send(b'\r');
            }
        }
        Ok(())
    }
}
