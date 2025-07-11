//! ATA Bus
//!
//! reference: https://wiki.osdev.org/IDE
//! reference: https://wiki.osdev.org/ATA_PIO_Mode
//! reference: https://github.com/theseus-os/Theseus/blob/HEAD/kernel/ata/src/lib.rs

use super::consts::*;
use alloc::boxed::Box;
use x86_64::instructions::port::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AtaBus {
    id: u8,              // 总线标识符（主/从通道）
    irq: u8,             // 中断号（实际未使用）
    io_base: u16,        // I/O端口基地址（数据寄存器组）
    ctrl_base: u16,      // 控制端口基地址（控制寄存器组）
    
    // 数据寄存器组
    data: Port<u16>,              // 数据端口(0x1F0)
    error: PortReadOnly<u8>,      // 错误寄存器(0x1F1)
    features: PortWriteOnly<u8>,  // 特性寄存器(0x1F1)
    sector_count: Port<u8>,       // 扇区计数(0x1F2)
    /// Also used for sector_number
    lba_low: Port<u8>,            // LBA低字节(0x1F3)
    /// Also used for cylinder_low
    lba_mid: Port<u8>,            // LBA中字节(0x1F4)
    /// Also used for cylinder_high
    lba_high: Port<u8>,           // LBA高字节(0x1F5)
    drive: Port<u8>,              // 驱动器选择(0x1F6)
    status: PortReadOnly<u8>,     // 状态寄存器(0x1F7)
    command: PortWriteOnly<u8>,   // 命令寄存器(0x1F7)
    
    // 控制寄存器组
    alternate_status: PortReadOnly<u8>,  // 替代状态(0x3F6)
    control: PortWriteOnly<u8>,         // 设备控制(0x3F6)
    drive_blockess: PortReadOnly<u8>,    // 驱动器地址(0x3F7)
}

impl AtaBus {
    pub fn new(id: u8, irq: u8, io_base: u16, ctrl_base: u16) -> Self { // 创建ATA总线实例
        Self {
            id,
            irq, // 实际未使用（使用轮询替代中断）
            io_base,
            ctrl_base,
            // 初始化所有端口寄存器
            data: Port::<u16>::new(io_base),             // 0x1F0
            error: PortReadOnly::<u8>::new(io_base + 1), // 0x1F1
            features: PortWriteOnly::<u8>::new(io_base + 1),
            sector_count: Port::<u8>::new(io_base + 2), // 0x1F2
            lba_low: Port::<u8>::new(io_base + 3),      // 0x1F3
            lba_mid: Port::<u8>::new(io_base + 4),       // 0x1F4
            lba_high: Port::<u8>::new(io_base + 5),     // 0x1F5
            drive: Port::<u8>::new(io_base + 6),         // 0x1F6
            status: PortReadOnly::new(io_base + 7),      // 0x1F7
            command: PortWriteOnly::new(io_base + 7),    // 0x1F7
            // 控制寄存器组
            alternate_status: PortReadOnly::new(ctrl_base),     // 0x3F6
            control: PortWriteOnly::new(ctrl_base),             // 0x3F6
            drive_blockess: PortReadOnly::new(ctrl_base + 1),   // 0x3F7
        }
    }

    #[inline]
    fn read_data(&mut self) -> u16 {
        unsafe { self.data.read() }
    }

    #[inline]
    fn write_data(&mut self, data: u16) {
        unsafe { self.data.write(data) }
    }

    /// Also used for LBAmid
    #[inline]
    fn cylinder_low(&mut self) -> u8 {
        unsafe { self.lba_mid.read() }
    }

    /// Also used for LBAhi
    #[inline]
    fn cylinder_high(&mut self) -> u8 {
        unsafe { self.lba_high.read() }
    }

    /// Reads the `status` port and returns the value as an `AtaStatus` bitfield.
    /// Because some buses operate (change wire values) very slowly,
    /// this undergoes the standard procedure of reading the alternate status port
    /// and discarding it 4 times before reading the real status port value.
    /// Each read is a 100ns delay, so the total delay of 400ns is proper.
    #[inline]
    fn status(&mut self) -> AtaStatus { // 获取设备当前状态
        AtaStatus::from_bits_truncate(unsafe {
            // wait for 400ns
            self.alternate_status.read();
            self.alternate_status.read();
            self.alternate_status.read();
            self.alternate_status.read();
            // read the status
            self.status.read()
        })
    }

    /// Reads the `error` port and returns the value as an `AtaError` bitfield.
    #[inline]
    fn error(&mut self) -> AtaError {
        AtaError::from_bits_truncate(unsafe { self.error.read() })
    }

    /// Returns true if the `status` port indicates an error.
    #[inline]
    fn is_error(&mut self) -> bool {
        self.status().contains(AtaStatus::ERROR)
    }

    /// Polls the `status` port until the given bit is set to the given value.
    #[inline]
    fn poll(&mut self, bit: AtaStatus, val: bool) { // 轮询方法，等待设备状态变更，等待bit的值变成val
        let mut status = self.status();
        // 持续检查直到目标状态位满足要求
        while status.intersects(bit) != val {
            // 发现错误立即调试
            if status.contains(AtaStatus::ERROR) {
                self.debug();
            }
            // 低功耗忙等待（避免占用100%CPU）
            core::hint::spin_loop();
            status = self.status();
        }
    }

    /// Log debug information about the bus
    fn debug(&mut self) {
        warn!("ATA error register  : {:?}", self.error());
        warn!("ATA status register : {:?}", self.status());
    }

    /// Writes the given command
    ///
    /// reference: https://wiki.osdev.org/ATA_PIO_Mode#28_bit_PIO
    fn write_command(&mut self, drive: u8, block: u32, cmd: AtaCommand) -> storage::FsResult {
        // drive: 设备选择（0=主设备，1=从设备）
        let bytes = block.to_le_bytes(); // a trick to convert u32 to [u8; 4]
        unsafe {
            // just 1 sector for current implementation
            self.sector_count.write(1);

            // FIXME: store the LBA28 address into four 8-bit registers
            //      - read the documentation for more information
            //      - enable LBA28 mode by setting the drive register
            self.lba_low.write(bytes[0]); // LBA low
            self.lba_mid.write(bytes[1]); // LBA mid
            self.lba_high.write(bytes[2]); // LBA high

            // 0-3是lba的24-27位，4是分辨主从盘，5、7必须为1，6是lba模式/chs模式
            // 构建drive寄存器值
            // let mut drive_val = 0xA0; // 二进制10100000 (位7,5=1 + LBA模式=1)
            
            // // 设置驱动器选择(位4)
            // if drive != 0 { // 假设drive=0表示主盘，非0表示从盘
            //     drive_val |= 0x10; // 设置位4为1
            // }
            
            // // 设置LBA最高4位(位3-0)
            // drive_val |= (bytes[3] & 0x0F); // 取block的第24-27位
            
            // // 写入drive寄存器
            // self.drive.write(drive_val);

            self.drive.write(0xE0 | ((drive & 1) << 4) | ((bytes[3] & 0x0F)));

            // FIXME: write the command register (cmd as u8)
            self.command.write(cmd as u8);
        }

        if self.status().is_empty() {
            // unknown drive
            return Err(storage::DeviceError::UnknownDevice.into());
        }

        // FIXME: poll for the status to be not BUSY
        self.poll(AtaStatus::BUSY, false);

        if self.is_error() {
            warn!("ATA error: {:?} command error", cmd);
            self.debug();
            return Err(storage::DeviceError::InvalidOperation.into());
        }

        // FIXME: poll for the status to be not BUSY and DATA_REQUEST_READY
        // self.poll(AtaStatus::BUSY | AtaStatus::DATA_REQUEST_READY, true);
        self.poll(AtaStatus::BUSY, false);
        self.poll(AtaStatus::DATA_REQUEST_READY,true);

        Ok(())
    }

    /// Identifies the drive at the given `drive` number (0 or 1).
    ///
    /// reference: https://wiki.osdev.org/ATA_PIO_Mode#IDENTIFY_command
    pub(super) fn identify_drive(&mut self, drive: u8) -> storage::FsResult<AtaDeviceType> {
        info!("Identifying drive {}", drive);

        // FIXME: use `AtaCommand::IdentifyDevice` to identify the drive
        //      - call `write_command` with `drive` and `0` as the block number
        //      - if the status is empty, return `AtaDeviceType::None`
        //      - else return `DeviceError::Unknown` as `FsError`
        self.write_command(drive, 0, AtaCommand::IdentifyDevice)?;

        // FIXME: poll for the status to be not BUSY
        self.poll(AtaStatus::BUSY, false);

        Ok(match (self.cylinder_low(), self.cylinder_high()) {
            // we only support PATA drives
            (0x00, 0x00) => AtaDeviceType::Pata(Box::new([0u16; 256].map(|_| self.read_data()))),
            // ignore the data as we don't support following types
            (0x14, 0xEB) => AtaDeviceType::PataPi,
            (0x3C, 0xC3) => AtaDeviceType::Sata,
            (0x69, 0x96) => AtaDeviceType::SataPi,
            _ => AtaDeviceType::None,
        })
    }

    /// Reads a block from the given drive and block number into the given buffer.
    ///
    /// reference: https://wiki.osdev.org/ATA_PIO_Mode#28_bit_PIO
    /// reference: https://wiki.osdev.org/IDE#Read.2FWrite_From_ATA_Drive
    pub(super) fn read_pio(
        &mut self,
        drive: u8,
        block: u32,
        buf: &mut [u8],
    ) -> storage::FsResult {
        self.write_command(drive, block, AtaCommand::ReadPio)?;

        // FIXME: read the data from the data port into the buffer
        //      - use `buf.chunks_mut(2)`
        //      - use `self.read_data()`
        //      - ! pay attention to data endianness
        for chunk in buf.chunks_mut(2) {
            // let data = self.read_data();
            // chunk[0] = (data & 0xFF) as u8; // lower byte
            // chunk[1] = (data >> 8) as u8;   // upper byte
            let data = self.read_data().to_le_bytes();
            chunk.copy_from_slice(&data);
        }

        if self.is_error() {
            debug!("ATA error: data read error");
            self.debug();
            Err(storage::DeviceError::ReadError.into())
        } else {
            Ok(())
        }
    }

    /// Writes a block to the given drive and block number from the given buffer.
    ///
    /// reference: https://wiki.osdev.org/ATA_PIO_Mode#28_bit_PIO
    /// reference: https://wiki.osdev.org/IDE#Read.2FWrite_From_ATA_Drive
    pub(super) fn write_pio(&mut self, drive: u8, block: u32, buf: &[u8]) -> storage::FsResult {
        self.write_command(drive, block, AtaCommand::WritePio)?;

        // FIXME: write the data from the buffer into the data port
        //      - use `buf.chunks(2)`
        //      - use `self.write_data()`
        //      - ! pay attention to data endianness
        for chunk in buf.chunks(2) {
            let data = u16::from_le_bytes(chunk.try_into().unwrap_or([0, 0]));
            self.write_data(data);
        }

        if self.is_error() {
            debug!("ATA error: data write error");
            self.debug();
            Err(storage::DeviceError::WriteError.into())
        } else {
            Ok(())
        }
    }
}
