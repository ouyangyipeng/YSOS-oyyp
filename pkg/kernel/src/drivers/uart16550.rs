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

            // 启用数据接收中断（IER 的 bit 0）
            PortWriteOnly::new(base + 1).write(0x01 as u8);
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


// /// ! 跟同学讨论后决定参考同学的方法重新实现串口（使用bitflag）
// /// 上面我原本写的那个也能用，但是感觉跟文档上说的有点不一样

// use bitflags::bitflags;
// use core::fmt;
// use x86_64::instructions::port::Port;
// /// A port-mapped UART 16550 serial interface.
// pub struct SerialPort {
//     data: Port<u8>,
//     interrupt_enable: Port<u8>,         //中断使能寄存器
//     interrupt_identification: Port<u8>, //中断标识寄存器
//     fifo_control: Port<u8>,             //FIFO 控制寄存器
//     line_control: Port<u8>,             //线路控制寄存器
//     modem_control: Port<u8>,            //调制解调器控制寄存器
//     line_status: Port<u8>,              //线路状态寄存器
//     modem_status: Port<u8>,             //调制解调器状态寄存器
//     scratch: Port<u8>,                  //暂存寄存器
// }

// bitflags! {
//     pub struct LineStatus: u8 {
//         const b1 = 0x00;
//         const b2 = 0x80;
//         const b3 = 0x03;
//         const b4 = 0xC7;
//         const b5 = 0x0B;
//         const b6 = 0x1E;
//         const b7 = 0xAE;
//         const b8 = 0x0F;
//     }
// }

// impl SerialPort {
//     pub const fn new(port: u16) -> Self {
//         SerialPort {
//             data: Port::new(port),
//             interrupt_enable: Port::new(port + 1),
//             interrupt_identification: Port::new(port + 2),
//             fifo_control: Port::new(port + 2),
//             line_control: Port::new(port + 3),
//             modem_control: Port::new(port + 4),
//             line_status: Port::new(port + 5),
//             modem_status: Port::new(port + 6),
//             scratch: Port::new(port + 7),
//         }
//     }

//     /// Initializes the serial port.
//     pub fn init(&mut self) {
//         // FIXME: Initialize the serial port
//         unsafe {         
//             self.interrupt_enable.write(LineStatus::b1.bits());
//             trace!("Serial port interrupt enable register: {:#x}", self.interrupt_enable.read());
//             self.line_control.write(LineStatus::b2.bits());
//             self.data.write(LineStatus::b3.bits());
//             self.interrupt_enable.write(LineStatus::b1.bits());
//             self.line_control.write(LineStatus::b3.bits());
//             self.fifo_control.write(LineStatus::b4.bits());
//             self.modem_control.write(LineStatus::b5.bits());
//             self.modem_control.write(LineStatus::b6.bits());
//             self.data.write(LineStatus::b7.bits());
//             if self.data.read() != LineStatus::b7.bits() {
//                 panic!("Serial port test failed: expected 0xAE, got {:#x}", self.data.read());
//             }
//             self.modem_control.write(LineStatus::b8.bits());
//             self.interrupt_enable.write(0x01);
//             trace!("Serial port interrupt enable register: {:#x}", self.interrupt_enable.read());
//         }
//     }

//     /// Sends a byte on the serial port.
//     pub fn send(&mut self, data: u8) {
//         // FIXME: Send a byte on the serial port
//         unsafe {
//             while self.line_status.read() & 0x20 == 0 {}
//             self.data.write(data);
//         }
//     }

//     /// Receives a byte on the serial port no wait.
//     pub fn receive(&mut self) -> Option<u8> {
//         // FIXME: Receive a byte on the serial port no wait
//         unsafe {
//             if self.line_status.read() & 1 == 0 {
//                 return None;
//             }
//             Some(self.data.read())
//         }
//     }
// }

// impl fmt::Write for SerialPort {
//     fn write_str(&mut self, s: &str) -> fmt::Result {
//         for byte in s.bytes() {
//             self.send(byte);
//         }
//         Ok(())
//     }
// }