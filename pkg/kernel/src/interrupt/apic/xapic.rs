use super::LocalApic;
use bit_field::BitField;
use core::fmt::{Debug, Error, Formatter};
use core::ptr::{read_volatile, write_volatile};
use x86::cpuid::CpuId;
use crate::interrupt::consts::{Interrupts, Irq};
use crate::memory::physical_to_virtual;
use bitflags::bitflags;

bitflags! {
    struct LvtFlags: u32 {
        // 向量号 (bits 0-7)
        const VECTOR_MASK = 0xFF;
        
        // 投递模式 (bits 8-10)
        const DELIVERY_MODE_BIT0 = 1 << 8;
        const DELIVERY_MODE_BIT1 = 1 << 9;
        const DELIVERY_MODE_BIT2 = 1 << 10;
        
        // 投递状态 (bit 12)
        const DELIVERY_STATUS = 1 << 12;
        
        // 中断屏蔽 (bit 16)
        const MASKED = 1 << 16;
        
        // 定时器模式 (bits 17-18)
        const TIMER_MODE_BIT0 = 1 << 17;
        const TIMER_MODE_BIT1 = 1 << 18;
        
        // 预定义的投递模式组合
        const DELIVERY_FIXED   = 0;                     // 000
        const DELIVERY_SMI     = Self::DELIVERY_MODE_BIT1.bits(); // 010
        const DELIVERY_NMI     = Self::DELIVERY_MODE_BIT2.bits(); // 100
        const DELIVERY_INIT    = Self::DELIVERY_MODE_BIT2.bits() | Self::DELIVERY_MODE_BIT0.bits(); // 101
        const DELIVERY_EXTINT  = Self::DELIVERY_MODE_BIT2.bits() | Self::DELIVERY_MODE_BIT1.bits() | Self::DELIVERY_MODE_BIT0.bits(); // 111
        
        // 定时器模式组合
        const TIMER_ONESHOT    = 0;                              // 00
        const TIMER_PERIODIC   = Self::TIMER_MODE_BIT0.bits();   // 01
        const TIMER_TSCDEADLINE= Self::TIMER_MODE_BIT1.bits();   // 10
    }
}

    
impl LvtFlags {
    // 辅助方法：设置投递模式
    fn set_delivery_mode(&mut self, mode: LvtFlags) {
        // 清除旧的投递模式位
        self.remove(
            Self::DELIVERY_MODE_BIT0 | 
            Self::DELIVERY_MODE_BIT1 | 
            Self::DELIVERY_MODE_BIT2
        );
        // 设置新的投递模式位
        self.insert(
            mode & (Self::DELIVERY_MODE_BIT0 | Self::DELIVERY_MODE_BIT1 | Self::DELIVERY_MODE_BIT2)
        );
    }
    
    // 辅助方法：设置定时器模式
    fn set_timer_mode(&mut self, mode: LvtFlags) {
        self.remove(Self::TIMER_MODE_BIT0 | Self::TIMER_MODE_BIT1);
        self.insert(mode & (Self::TIMER_MODE_BIT0 | Self::TIMER_MODE_BIT1));
    }
}

/// Default physical address of xAPIC
pub const LAPIC_ADDR: u64 = 0xFEE00000;

pub struct XApic {
    addr: u64,
}

impl XApic {
    pub unsafe fn new(addr: u64) -> Self {
        // info!("XApic::new: addr = {:#x}", addr);
        XApic { addr }// 在这里映射更加安全吧？
        // let addr = physical_to_virtual(addr);
        // XApic { addr }
        // 本来就不应该在外部映射完再传入
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        unsafe {// 这个unsafe也没必要吧？
            // 是不是应该内存对齐一下？
            // assert!(reg % 4 == 0, "APIC register offset must be 32-bit aligned");
            // assert!(reg <= 0x3FF, "APIC register offset out of range");
            read_volatile((self.addr + reg as u64) as *const u32)
        }
    }

    unsafe fn write(&mut self, reg: u32, value: u32) {
        unsafe {
            write_volatile((self.addr + reg as u64) as *mut u32, value);
            self.read(0x20);// 这行到底是干啥的？？没看懂
            // 如果要确保写入完成，应该使用内存屏障
            // core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        }
    }
}

impl LocalApic for XApic {
    /// If this type APIC is supported
    fn support() -> bool {
        // FIXME: Check CPUID to see if xAPIC is supported.
        // let cpuid = CpuId::new();
        // // CPUID.01h:EDX.APIC[bit 9]
        // let eax = cpuid.get_cpuid(0x1).unwrap();
        // let edx = eax.edx();
        // if edx.get_bit(9) {
        //     true
        // }else {
        //     false
        // }
        // 以上是最开始尝试CPUID.01h:EDX.APIC[bit 9]里面获取
        CpuId::new()
            .get_feature_info()
            .map(|f| f.has_apic())
            .unwrap_or(false)
    }

    /// Initialize the xAPIC for the current CPU.
    fn cpu_init(&mut self) {
        unsafe {
            // ! FIXME: Enable local APIC; set spurious interrupt vector.
            let mut spiv = self.read(0xF0);
            spiv |= 1 << 8; // set EN bit
            // clear and set Vector
            spiv &= !(0xFF);
            spiv |= Interrupts::IrqBase as u32 + Irq::Spurious as u32;
            self.write(0xF0, spiv);

            // ! FIXME: The timer repeatedly counts down at bus frequency
            // 设置定时器的分频系数
            // self.write(0x3E0, 0b1011); // set Timer Divide to 1
            self.write(0x3E0, 0b1010); // set Timer Divide to 128
            self.write(0x380, 0x80000); // set initial count to 0x80000

            // lvt reg
            let mut lvt_timer = self.read(0x320);
            // clear and set Vector
            lvt_timer &= !(0xFF);
            lvt_timer |= Interrupts::IrqBase as u32 + Irq::Timer as u32;
            lvt_timer &= !(1 << 16); // clear Mask
            lvt_timer |= 1 << 17; // set Timer Periodic Mode
            self.write(0x320, lvt_timer);

            // ! FIXME: Disable logical interrupt lines (LINT0, LINT1)
            self.write(0x350, 1 << 16); // lint0
            self.write(0x360, 1 << 16); // lint1

            // ! FIXME: Disable performance counter overflow interrupts (PCINT)
            self.write(0x340, 1 << 16);

            // ! FIXME: Map error interrupt to IRQ_ERROR.
            // let mut lvt_error = self.read(0x370);
            // // clear and set Vector
            // lvt_error &= !(0xFF);
            // lvt_error |= Interrupts::IrqBase as u32 + Irq::Error as u32;
            // self.write(0x370, lvt_error);

            // 尝试用bitflags!宏来设置标志位
            let mut lvt_error = self.read(0x370);
            let mut lvt_flags = LvtFlags::from_bits_truncate(lvt_error);
            // 清除旧的向量号并设置新的向量号
            lvt_flags.remove(LvtFlags::VECTOR_MASK);
            lvt_flags.insert(LvtFlags::from_bits_truncate(
                Interrupts::IrqBase as u32 + Irq::Error as u32
            ));
            self.write(0x370, lvt_flags.bits());

            // ! FIXME: Clear error status register (requires back-to-back writes).
            self.write(0x280, 0);
            self.write(0x280, 0);
            // 连写两次确保清楚错误

            // ! FIXME: Ack any outstanding interrupts.
            self.write(0x0B0, 0);
            // 发送EOI信号，结束当前中断
            // 也可以self.eoi()来实现
            // self.eoi();

            // ! FIXME: Send an Init Level De-Assert to synchronise arbitration ID's.
            self.write(0x310, 0); // set ICR 0x310
            const BCAST: u32 = 1 << 19;
            const INIT: u32 = 5 << 8;
            const TMLV: u32 = 1 << 15; // TM = 1, LV = 0
            self.write(0x300, BCAST | INIT | TMLV); // set ICR 0x300
            const DS: u32 = 1 << 12;
            while self.read(0x300) & DS != 0 {} // wait for delivery status

            // ! FIXME: Enable interrupts on the APIC (but not on the processor).
            // tpr寄存器设为0x00，允许所有中断
            self.write(0x80, 0);
        }

        // NOTE: Try to use bitflags! macro to set the flags.
    }

    fn id(&self) -> u32 {
        // NOTE: Maybe you can handle regs like `0x0300` as a const.
        unsafe { self.read(0x0020) >> 24 }
    }

    fn version(&self) -> u32 {
        unsafe { self.read(0x0030) }
    }

    fn icr(&self) -> u64 {
        unsafe { (self.read(0x0310) as u64) << 32 | self.read(0x0300) as u64 }
    }

    fn set_icr(&mut self, value: u64) {
        unsafe {
            while self.read(0x0300).get_bit(12) {}
            self.write(0x0310, (value >> 32) as u32);
            self.write(0x0300, value as u32);
            while self.read(0x0300).get_bit(12) {}
        }
    }

    fn eoi(&mut self) {
        unsafe {
            self.write(0x00B0, 0);
        }
    }
}

impl Debug for XApic {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("Xapic")
            .field("id", &self.id())
            .field("version", &self.version())
            .field("icr", &self.icr())
            .finish()
    }
}
