//! ATA Drive
//!
//! reference: https://wiki.osdev.org/IDE
//! reference: https://wiki.osdev.org/ATA_PIO_Mode
//! reference: https://github.com/theseus-os/Theseus/blob/HEAD/kernel/ata/src/lib.rs

mod bus;
mod consts;

use alloc::{boxed::Box, string::String};
use bus::AtaBus;
use consts::AtaDeviceType;
use spin::Mutex;

lazy_static! {
    pub static ref BUSES: [Mutex<AtaBus>; 2] = {
        let buses = [
            Mutex::new(AtaBus::new(0, 14, 0x1F0, 0x3F6)),
            Mutex::new(AtaBus::new(1, 15, 0x170, 0x376)),
        ];

        info!("Initialized ATA Buses.");

        buses
    };
}

// 根据文档
pub const ATA_IDENT_SERIAL:usize = 20 ;  // 20 bytes
pub const ATA_IDENT_SERIAL_SIZE:usize = 20;
pub const ATA_IDENT_MODEL:usize = 54;   // 40 bytes
pub const ATA_IDENT_MODEL_SIZE:usize = 40;
pub const ATA_IDENT_MAX_LBA:usize = 120; // 4 bytes (unsigned int)
pub const ATA_IDENT_MAX_LBA_SIZE:usize = 4;

#[derive(Clone)]
pub struct AtaDrive {
    pub bus: u8,
    pub drive: u8,
    blocks: u32,
    model: Box<str>,
    serial: Box<str>,
}

impl AtaDrive {
    pub fn open(bus: u8, drive: u8) -> Option<Self> {
        trace!("Opening drive {}@{}...", bus, drive);

        // we only support PATA drives
        if let Ok(AtaDeviceType::Pata(res)) = BUSES[bus as usize].lock().identify_drive(drive) {
            let buf = res.map(u16::to_be_bytes).concat();
            let serial = { /* FIXME: get the serial from buf */ 
                String::from_utf8_lossy(
                    &buf[ATA_IDENT_SERIAL..ATA_IDENT_SERIAL + ATA_IDENT_SERIAL_SIZE],
                )
                // .trim_end_matches('\0')
                .trim()
                .into()
            };
            let model = { /* FIXME: get the model from buf */ 
                String::from_utf8_lossy(
                    &buf[ATA_IDENT_MODEL..ATA_IDENT_MODEL + ATA_IDENT_MODEL_SIZE],
                )
                // .trim_end_matches('\0')
                .trim()
                .into()
            };
            let blocks = { /* FIXME: get the block count from buf */ 
                u32::from_be_bytes(
                    buf[ATA_IDENT_MAX_LBA..ATA_IDENT_MAX_LBA + ATA_IDENT_MAX_LBA_SIZE]
                        .try_into()
                        .unwrap_or([0; ATA_IDENT_MAX_LBA_SIZE])
                )
                .rotate_left(16)
            };
            let ata_drive = Self {
                bus,
                drive,
                model,
                serial,
                blocks,
            };
            info!("Drive {} opened", ata_drive);
            Some(ata_drive)
        } else {
            warn!("Drive {}@{} is not a PATA drive", bus, drive);
            None
        }
    }

    fn humanized_size(&self) -> (f32, &'static str) {
        let size = self.block_size();
        let count = self.block_count().unwrap();
        let bytes = size * count;

        // 这里有点问题
        // crate::humanized_size(bytes as u64)
        let result = crate::humanized_size(bytes as u64);
        (result.0 as f32,result.1)
    }
}

impl core::fmt::Display for AtaDrive {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (size, unit) = self.humanized_size();
        write!(f, "{} {} ({} {})", self.model, self.serial, size, unit)
    }
}

use storage::{Block512, BlockDevice};

impl BlockDevice<Block512> for AtaDrive {
    fn block_count(&self) -> storage::FsResult<usize> {
        // FIXME: return the block count
        Ok(self.blocks as usize)
    }

    fn read_block(&self, offset: usize, block: &mut Block512) -> storage::FsResult {
        // FIXME: read the block
        //      - use `BUSES` and `self` to get bus
        //      - use `read_pio` to get data
        BUSES[self.bus as usize]
            .lock()
            .read_pio(
                self.drive, 
                offset as u32, 
                block.as_mut()
            )
            // .map_err(|e| e.into())
            // .and_then(|_| {
            //     if block.len() != 512 {
            //         Err(storage::DeviceError::ReadError.into())
            //     } else {
            //         Ok(())
            //     }
            // })
    }

    fn write_block(&self, offset: usize, block: &Block512) -> storage::FsResult {
        // FIXME: write the block
        //      - use `BUSES` and `self` to get bus
        //      - use `write_pio` to write data
        BUSES[self.bus as usize]
            .lock()
            .write_pio(
                self.drive,
                offset as u32,
                block.as_ref()
            )
            // .map_err(|e| e.into())
            // .and_then(|_| {
            //     if block.len() != 512 {
            //         Err(storage::DeviceError::WriteError.into())
            //     } else {
            //         Ok(())
            //     }
            // })
    }
}
