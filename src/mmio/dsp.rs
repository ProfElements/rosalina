use bit_field::BitField;
use voladdress::{Safe, VolAddress};

use super::{
    exi::DmaStart,
    pi::{InterruptState, Mask},
    vi::{Enabled, Reset},
};

pub const BASE: usize = 0xCC00_5000;

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MailboxHi(u16);

pub const DSP_MAIL_HI: VolAddress<MailboxHi, Safe, Safe> = unsafe { VolAddress::new(BASE) };

pub const CPU_MAIL_HI: VolAddress<MailboxHi, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x4) };

impl From<u16> for MailboxHi {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<MailboxHi> for u16 {
    fn from(value: MailboxHi) -> Self {
        value.0
    }
}

impl MailboxHi {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_dsp() -> Self {
        DSP_MAIL_HI.read()
    }

    pub fn read_cpu() -> Self {
        CPU_MAIL_HI.read()
    }

    pub fn write_dsp(self) {
        DSP_MAIL_HI.write(self);
    }

    pub fn write_cpu(self) {
        CPU_MAIL_HI.write(self);
    }

    pub fn mailbox_status(&self) -> DmaStart {
        self.0.get_bit(15).into()
    }

    pub fn with_mailbox_status(&mut self, start: DmaStart) -> &mut Self {
        self.0.set_bit(15, start.into());
        self
    }

    pub fn data(&self) -> u16 {
        self.0.get_bits(0..=14)
    }

    pub fn with_data(&mut self, data: u16) -> &mut Self {
        debug_assert!(data < (2 ^ 15), "Data must be less then 2147483648");
        self.0.set_bits(0..=14, data);
        self
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MailboxLow(u16);

pub const DSP_MAIL_LO: VolAddress<MailboxLow, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x2) };

pub const CPU_MAIL_LO: VolAddress<MailboxLow, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x6) };

impl From<u16> for MailboxLow {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<MailboxLow> for u16 {
    fn from(value: MailboxLow) -> Self {
        value.0
    }
}

impl MailboxLow {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_dsp() -> Self {
        DSP_MAIL_LO.read()
    }

    pub fn read_cpu() -> Self {
        CPU_MAIL_LO.read()
    }

    pub fn write_dsp(self) {
        DSP_MAIL_LO.write(self);
    }

    pub fn write_cpu(self) {
        CPU_MAIL_LO.write(self);
    }

    pub fn data(&self) -> u16 {
        self.0.get_bits(0..=15)
    }

    pub fn with_data(&mut self, data: u16) -> &mut Self {
        self.0.set_bits(0..=15, data);
        self
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct DspControl(u16);

pub const DSP_CONTROL: VolAddress<DspControl, Safe, Safe> = unsafe { VolAddress::new(BASE + 0xA) };

impl From<u16> for DspControl {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<DspControl> for u16 {
    fn from(value: DspControl) -> Self {
        value.0
    }
}

impl DspControl {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        DSP_CONTROL.read()
    }

    pub fn write(self) {
        DSP_CONTROL.write(self);
    }

    pub fn reset(&self) -> Reset {
        self.0.get_bit(0).into()
    }

    pub fn with_reset(&mut self, reset: Reset) -> &mut Self {
        self.0.set_bit(0, reset.into());
        self
    }

    pub fn assert_interrupt(&self) -> InterruptState {
        self.0.get_bit(1).into()
    }

    pub fn with_assert_interrupt(&mut self, assert: InterruptState) -> &mut Self {
        self.0.set_bit(1, assert.into());
        self
    }

    pub fn halt(&self) -> Halt {
        self.0.get_bit(2).into()
    }

    pub fn with_halt(&mut self, halt: Halt) -> &mut Self {
        self.0.set_bit(2, halt.into());
        self
    }

    pub fn dma_interrupt(&self) -> InterruptState {
        self.0.get_bit(3).into()
    }

    pub fn with_dma_interrupt(&mut self, state: InterruptState) -> &mut Self {
        self.0.set_bit(3, state.into());
        self
    }

    pub fn dma_interrupt_mask(&self) -> Mask {
        self.0.get_bit(4).into()
    }

    pub fn with_dma_interrupt_mask(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(4, mask.into());
        self
    }

    pub fn aram_interrupt(&self) -> InterruptState {
        self.0.get_bit(5).into()
    }

    pub fn with_aram_interrupt(&mut self, state: InterruptState) -> &mut Self {
        self.0.set_bit(5, state.into());
        self
    }

    pub fn aram_interrupt_mask(&self) -> Mask {
        self.0.get_bit(6).into()
    }

    pub fn with_aram_interrupt_mask(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(6, mask.into());
        self
    }

    pub fn dsp_interrupt(&self) -> InterruptState {
        self.0.get_bit(7).into()
    }

    pub fn with_dsp_interrupt(&mut self, state: InterruptState) -> &mut Self {
        self.0.set_bit(7, state.into());
        self
    }

    pub fn dsp_interrupt_mask(&self) -> Mask {
        self.0.get_bit(8).into()
    }

    pub fn with_dsp_interrupt_mask(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(8, mask.into());
        self
    }

    pub fn dma_status(&self) -> DmaStart {
        self.0.get_bit(9).into()
    }

    pub fn with_dma_status(&mut self, dma: DmaStart) -> &mut Self {
        self.0.set_bit(9, dma.into());
        self
    }

    pub fn inited1(&self) -> bool {
        self.0.get_bit(10)
    }

    pub fn dsp_enabled(&self) -> Enabled {
        self.0.get_bit(11).into()
    }

    pub fn with_dsp_enabled(&mut self, enable: Enabled) -> &mut Self {
        self.0.set_bit(11, enable.into());
        self
    }
}

pub enum Halt {
    Halted,
    Idle,
}

impl From<bool> for Halt {
    fn from(value: bool) -> Self {
        if value {
            Self::Halted
        } else {
            Self::Idle
        }
    }
}

impl From<Halt> for bool {
    fn from(value: Halt) -> Self {
        match value {
            Halt::Halted => true,
            Halt::Idle => false,
        }
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AramSize(u16);

pub const ARAM_SIZE: VolAddress<AramSize, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x12) };

impl From<u16> for AramSize {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<AramSize> for u16 {
    fn from(value: AramSize) -> Self {
        value.0
    }
}

impl AramSize {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        ARAM_SIZE.read()
    }

    pub fn write(self) {
        ARAM_SIZE.write(self);
    }

    pub fn size(&self) -> u16 {
        self.0.get_bits(0..=15)
    }

    pub fn with_size(&mut self, size: u16) -> &mut Self {
        self.0.set_bits(0..=15, size);
        self
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AramMode(u16);

pub const ARAM_MODE: VolAddress<AramMode, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x16) };

impl From<u16> for AramMode {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<AramMode> for u16 {
    fn from(value: AramMode) -> Self {
        value.0
    }
}

impl AramMode {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        ARAM_MODE.read()
    }

    pub fn write(self) {
        ARAM_MODE.write(self);
    }

    pub fn mode(&self) -> u16 {
        self.0.get_bits(0..=15)
    }

    pub fn with_mode(&mut self, mode: u16) -> &mut Self {
        self.0.set_bits(0..=15, mode);
        self
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AramRefresh(u16);

pub const ARAM_REFRESH: VolAddress<AramRefresh, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x1A) };

impl From<u16> for AramRefresh {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<AramRefresh> for u16 {
    fn from(value: AramRefresh) -> Self {
        value.0
    }
}

impl AramRefresh {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        ARAM_REFRESH.read()
    }

    pub fn write(self) {
        ARAM_REFRESH.write(self);
    }

    pub fn refresh(&self) -> u16 {
        self.0.get_bits(0..=15)
    }

    pub fn with_refresh(&mut self, refresh: u16) -> &mut Self {
        self.0.set_bits(0..=15, refresh);
        self
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AddrHi(u16);

pub const MAIN_MEM_ADDR_HI: VolAddress<AddrHi, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x20) };

pub const ARAM_MEM_ADDR_HI: VolAddress<AddrHi, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x24) };

impl From<u16> for AddrHi {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<AddrHi> for u16 {
    fn from(value: AddrHi) -> Self {
        value.0
    }
}

impl AddrHi {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_main_mem() -> Self {
        MAIN_MEM_ADDR_HI.read()
    }

    pub fn read_audio_mem() -> Self {
        ARAM_MEM_ADDR_HI.read()
    }

    pub fn write_main_mem(self) {
        MAIN_MEM_ADDR_HI.write(self);
    }

    pub fn write_audio_mem(self) {
        ARAM_MEM_ADDR_HI.write(self);
    }

    pub fn addr_high(&self) -> u16 {
        self.0.get_bits(0..=15)
    }

    pub fn with_addr_high(&mut self, high: u16) -> &mut Self {
        self.0.set_bits(0..=15, high);
        self
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AddrLo(u16);

pub const MAIN_MEM_ADDR_LO: VolAddress<AddrLo, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x22) };

pub const ARAM_MEM_ADDR_LO: VolAddress<AddrLo, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x26) };

impl From<u16> for AddrLo {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<AddrLo> for u16 {
    fn from(value: AddrLo) -> Self {
        value.0
    }
}

impl AddrLo {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_main_mem() -> Self {
        MAIN_MEM_ADDR_LO.read()
    }

    pub fn read_audio_mem() -> Self {
        ARAM_MEM_ADDR_LO.read()
    }

    pub fn write_main_mem(self) {
        MAIN_MEM_ADDR_LO.write(self);
    }

    pub fn write_audio_mem(self) {
        ARAM_MEM_ADDR_LO.write(self);
    }

    pub fn addr_low(&self) -> u16 {
        self.0.get_bits(0..=15)
    }

    pub fn with_addr_low(&mut self, low: u16) -> &mut Self {
        self.0.set_bits(0..=15, low);
        self
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AramDmaCountHi(u16);

pub const ARAM_DMA_COUNT_HI: VolAddress<AramDmaCountHi, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x28) };

impl From<u16> for AramDmaCountHi {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<AramDmaCountHi> for u16 {
    fn from(value: AramDmaCountHi) -> Self {
        value.0
    }
}

impl AramDmaCountHi {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        ARAM_DMA_COUNT_HI.read()
    }

    pub fn write(self) {
        ARAM_DMA_COUNT_HI.write(self);
    }

    pub fn dma_type(&self) -> DmaType {
        self.0.get_bit(15).into()
    }

    pub fn with_dma_type(&mut self, kind: DmaType) -> &mut Self {
        self.0.set_bit(15, kind.into());
        self
    }

    pub fn count_hi(&self) -> u16 {
        self.0.get_bits(0..=14)
    }

    pub fn with_count_hi(&mut self, count: u16) -> &mut Self {
        debug_assert!(count < 32768, "count high must be less 32768");
        self.0.set_bits(0..=14, count);
        self
    }
}

pub enum DmaType {
    Read,
    Write,
}

impl From<bool> for DmaType {
    fn from(value: bool) -> Self {
        if value {
            Self::Read
        } else {
            Self::Write
        }
    }
}

impl From<DmaType> for bool {
    fn from(value: DmaType) -> Self {
        match value {
            DmaType::Read => true,
            DmaType::Write => false,
        }
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AramDmaCountLo(u16);

pub const ARAM_DMA_COUNT_LO: VolAddress<AramDmaCountLo, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x2A) };

impl From<u16> for AramDmaCountLo {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<AramDmaCountLo> for u16 {
    fn from(value: AramDmaCountLo) -> Self {
        value.0
    }
}

impl AramDmaCountLo {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        ARAM_DMA_COUNT_LO.read()
    }

    pub fn write(self) {
        ARAM_DMA_COUNT_LO.write(self);
    }

    pub fn count_low(&self) -> u16 {
        self.0.get_bits(0..=15)
    }

    pub fn with_count_low(&mut self, count: u16) -> &mut Self {
        self.0.set_bits(0..=15, count);
        self
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AudioDmaAddrHi(u16);

pub const AUDIO_DMA_ADDR_HI: VolAddress<AudioDmaAddrHi, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x30) };

impl From<u16> for AudioDmaAddrHi {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<AudioDmaAddrHi> for u16 {
    fn from(value: AudioDmaAddrHi) -> Self {
        value.0
    }
}

impl AudioDmaAddrHi {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        AUDIO_DMA_ADDR_HI.read()
    }

    pub fn write(self) {
        AUDIO_DMA_ADDR_HI.write(self);
    }

    pub fn addr_high(&self) -> u16 {
        self.0.get_bits(0..=15)
    }

    pub fn with_addr_high(&mut self, addr_high: u16) -> &mut Self {
        self.0.set_bits(0..=15, addr_high);
        self
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AudioDmaAddrLo(u16);

pub const AUDIO_DMA_ADDR_LO: VolAddress<AudioDmaAddrLo, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x32) };

impl From<u16> for AudioDmaAddrLo {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<AudioDmaAddrLo> for u16 {
    fn from(value: AudioDmaAddrLo) -> Self {
        value.0
    }
}

impl AudioDmaAddrLo {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        AUDIO_DMA_ADDR_LO.read()
    }

    pub fn write(self) {
        AUDIO_DMA_ADDR_LO.write(self);
    }

    pub fn addr_low(&self) -> u16 {
        self.0.get_bits(0..=15)
    }

    pub fn with_addr_low(&mut self, addr_high: u16) -> &mut Self {
        self.0.set_bits(0..=15, addr_high);
        self
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AudioDmaControl(u16);

pub const AUDIO_DMA_CONTROL: VolAddress<AudioDmaControl, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x36) };

impl From<u16> for AudioDmaControl {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<AudioDmaControl> for u16 {
    fn from(value: AudioDmaControl) -> Self {
        value.0
    }
}

impl AudioDmaControl {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        AUDIO_DMA_CONTROL.read()
    }

    pub fn write(self) {
        AUDIO_DMA_CONTROL.write(self);
    }

    pub fn dma_block_count(&self) -> u16 {
        self.0.get_bits(0..=14)
    }

    pub fn with_dma_block_count(&mut self, count: u16) -> &mut Self {
        debug_assert!(count < 32768, "Dma block count must be less then 32768");
        self.0.set_bits(0..=14, count);
        self
    }

    pub fn dma_start(&self) -> DmaStart {
        self.0.get_bit(15).into()
    }

    pub fn with_dma_start(&mut self, dma_start: DmaStart) -> &mut Self {
        self.0.set_bit(15, dma_start.into());
        self
    }
}
