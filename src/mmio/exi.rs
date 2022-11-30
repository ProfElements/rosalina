use bit_field::BitField;
use voladdress::{Safe, VolAddress};

use super::pi::{InterruptState, Mask};

pub const BASE: usize = 0xCD00_6800;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct ExiParams(u32);

pub const EXI_CHANNEL_0_PARAMS: VolAddress<ExiParams, Safe, Safe> =
    unsafe { VolAddress::new(BASE) };

pub const EXI_CHANNEL_1_PARAMS: VolAddress<ExiParams, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x14) };

pub const EXI_CHANNEL_2_PARAMS: VolAddress<ExiParams, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x28) };

impl From<u32> for ExiParams {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<ExiParams> for u32 {
    fn from(value: ExiParams) -> Self {
        value.0
    }
}

#[repr(u8)]
pub enum ExiClock {
    OneMegahertz,
    TwoMegahertz,
    FourMegahertz,
    EightMegahertz,
    SixteenMegahertz,
    ThirtyTwoMegahertz,
}

#[derive(Debug)]
pub struct InvalidExiClockError;

impl TryFrom<u32> for ExiClock {
    type Error = InvalidExiClockError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::OneMegahertz),
            1 => Ok(Self::TwoMegahertz),
            2 => Ok(Self::FourMegahertz),
            3 => Ok(Self::EightMegahertz),
            4 => Ok(ExiClock::SixteenMegahertz),
            5 => Ok(Self::ThirtyTwoMegahertz),
            _ => Err(InvalidExiClockError),
        }
    }
}

impl From<ExiClock> for u32 {
    fn from(value: ExiClock) -> Self {
        match value {
            ExiClock::OneMegahertz => 0,
            ExiClock::TwoMegahertz => 1,
            ExiClock::FourMegahertz => 2,
            ExiClock::EightMegahertz => 3,
            ExiClock::SixteenMegahertz => 4,
            ExiClock::ThirtyTwoMegahertz => 5,
        }
    }
}

pub enum ExiDevice {
    None,
    Device0,
    Device1,
    Device2,
}

#[derive(Debug)]
pub struct InvalidExiDeviceError;

impl TryFrom<u32> for ExiDevice {
    type Error = InvalidExiDeviceError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0b000 => Ok(Self::None),
            0b001 => Ok(Self::Device0),
            0b010 => Ok(Self::Device1),
            0b100 => Ok(Self::Device2),
            _ => Err(InvalidExiDeviceError),
        }
    }
}

impl From<ExiDevice> for u32 {
    fn from(value: ExiDevice) -> Self {
        match value {
            ExiDevice::None => 0b000,
            ExiDevice::Device0 => 0b001,
            ExiDevice::Device1 => 0b010,
            ExiDevice::Device2 => 0b100,
        }
    }
}

pub enum DeviceConnected {
    Connected,
    Disconnected,
}

impl From<bool> for DeviceConnected {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Connected,
            false => Self::Disconnected,
        }
    }
}

impl From<DeviceConnected> for bool {
    fn from(value: DeviceConnected) -> Self {
        match value {
            DeviceConnected::Connected => true,
            DeviceConnected::Disconnected => false,
        }
    }
}

pub enum RomDiscramble {
    Discramble,
    Disabled,
}

impl From<bool> for RomDiscramble {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Disabled,
            false => Self::Discramble,
        }
    }
}

impl From<RomDiscramble> for bool {
    fn from(value: RomDiscramble) -> Self {
        match value {
            RomDiscramble::Disabled => true,
            RomDiscramble::Discramble => false,
        }
    }
}

impl ExiParams {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_zero() -> Self {
        EXI_CHANNEL_0_PARAMS.read()
    }

    pub fn read_one() -> Self {
        EXI_CHANNEL_1_PARAMS.read()
    }

    pub fn read_two() -> Self {
        EXI_CHANNEL_2_PARAMS.read()
    }

    pub fn write_zero(self) {
        EXI_CHANNEL_0_PARAMS.write(self)
    }

    pub fn write_one(self) {
        EXI_CHANNEL_1_PARAMS.write(self)
    }

    pub fn write_two(self) {
        EXI_CHANNEL_2_PARAMS.write(self)
    }

    pub fn exi_interrupt_mask(&self) -> Mask {
        self.0.get_bit(0).into()
    }

    pub fn with_exi_interrupt_mask(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(0, mask.into());
        self
    }

    pub fn exi_status(&self) -> InterruptState {
        self.0.get_bit(1).into()
    }

    pub fn with_exi_status(&mut self, status: InterruptState) -> &mut Self {
        self.0.set_bit(1, status.into());
        self
    }

    pub fn transfer_complete_mask(&self) -> Mask {
        self.0.get_bit(2).into()
    }

    pub fn with_transfer_complete_mask(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(2, mask.into());
        self
    }

    pub fn transfer_complete_status(&self) -> InterruptState {
        self.0.get_bit(3).into()
    }

    pub fn with_transfer_complete_status(&mut self, status: InterruptState) -> &mut Self {
        self.0.set_bit(3, status.into());
        self
    }

    pub fn clock(&self) -> ExiClock {
        self.0.get_bits(4..=6).try_into().unwrap()
    }

    pub fn with_clock(&mut self, clock: ExiClock) -> &mut Self {
        self.0.set_bits(4..=6, clock.into());
        self
    }

    pub fn device_select(&self) -> ExiDevice {
        self.0.get_bits(7..=9).try_into().unwrap()
    }

    pub fn with_device_select(&mut self, device: ExiDevice) -> &mut Self {
        self.0.set_bits(7..=9, device.into());
        self
    }

    pub fn external_insertion_mask(&self) -> Mask {
        self.0.get_bit(10).into()
    }

    pub fn with_external_insertion_mask(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(10, mask.into());
        self
    }

    pub fn external_insertion_status(&self) -> InterruptState {
        self.0.get_bit(11).into()
    }

    pub fn with_external_insertion_status(&mut self, status: InterruptState) -> &mut Self {
        self.0.set_bit(11, status.into());
        self
    }

    pub fn device_connected(&self) -> DeviceConnected {
        self.0.get_bit(12).into()
    }

    pub fn wtih_device_connected(&mut self, connected: DeviceConnected) -> &mut Self {
        self.0.set_bit(12, connected.into());
        self
    }

    pub fn rom_discramble(&self) -> RomDiscramble {
        self.0.get_bit(13).into()
    }

    pub fn with_rom_discramble(&mut self, discramble: RomDiscramble) -> &mut Self {
        self.0.set_bit(13, discramble.into());
        self
    }

    pub fn has_interrupts(&self) -> bool {
        self.exi_status() == InterruptState::Happened
            || self.transfer_complete_status() == InterruptState::Happened
            || self.external_insertion_status() == InterruptState::Happened
    }
}

pub const EXI_CHANNEL_0_DMA_START: VolAddress<usize, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x4) };

pub const EXI_CHANNEL_1_DMA_START: VolAddress<usize, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x18) };

pub const EXI_CHANNEL_2_DMA_START: VolAddress<usize, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x2C) };

pub const EXI_CHANNEL_0_DMA_LENGTH: VolAddress<usize, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x8) };

pub const EXI_CHANNEL_1_DMA_LENGTH: VolAddress<usize, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x1C) };

pub const EXI_CHANNEL_2_DMA_LENGTH: VolAddress<usize, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x30) };

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct ExiControl(u32);

pub const EXI_CHANNEL_0_CONTROL: VolAddress<ExiControl, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0xC) };

pub const EXI_CHANNEL_1_CONTROL: VolAddress<ExiControl, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x20) };

pub const EXI_CHANNEL_2_CONTROL: VolAddress<ExiControl, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x34) };

impl From<u32> for ExiControl {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<ExiControl> for u32 {
    fn from(value: ExiControl) -> Self {
        value.0
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum DmaStart {
    Start,
    Idle,
}

impl From<bool> for DmaStart {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Start,
            false => Self::Idle,
        }
    }
}

impl From<DmaStart> for bool {
    fn from(value: DmaStart) -> Self {
        match value {
            DmaStart::Start => true,
            DmaStart::Idle => false,
        }
    }
}

pub enum DmaMode {
    Immediate,
    Dma,
}

impl From<bool> for DmaMode {
    fn from(value: bool) -> Self {
        match value {
            true => DmaMode::Dma,
            false => DmaMode::Immediate,
        }
    }
}

impl From<DmaMode> for bool {
    fn from(value: DmaMode) -> Self {
        match value {
            DmaMode::Immediate => false,
            DmaMode::Dma => true,
        }
    }
}

pub enum ReadWriteMode {
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug)]
pub struct InvalidReadWriteModeError;

impl TryFrom<u32> for ReadWriteMode {
    type Error = InvalidReadWriteModeError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(Self::Read),
            0b01 => Ok(Self::Write),
            0b10 => Ok(Self::ReadWrite),
            _ => Err(InvalidReadWriteModeError),
        }
    }
}

impl From<ReadWriteMode> for u32 {
    fn from(value: ReadWriteMode) -> Self {
        match value {
            ReadWriteMode::Read => 0b00,
            ReadWriteMode::Write => 0b01,
            ReadWriteMode::ReadWrite => 0b10,
        }
    }
}

impl ExiControl {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_zero() -> Self {
        EXI_CHANNEL_0_CONTROL.read()
    }

    pub fn read_one() -> Self {
        EXI_CHANNEL_1_CONTROL.read()
    }

    pub fn read_two() -> Self {
        EXI_CHANNEL_2_CONTROL.read()
    }

    pub fn write_zero(self) {
        EXI_CHANNEL_0_CONTROL.write(self)
    }

    pub fn write_one(self) {
        EXI_CHANNEL_1_CONTROL.write(self)
    }

    pub fn write_two(self) {
        EXI_CHANNEL_2_CONTROL.write(self)
    }

    pub fn dma_start(&self) -> DmaStart {
        self.0.get_bit(0).into()
    }

    pub fn with_dma_start(&mut self, start: DmaStart) -> &mut Self {
        self.0.set_bit(0, start.into());
        self
    }

    pub fn dma_mode(&self) -> DmaMode {
        self.0.get_bit(1).into()
    }

    pub fn with_dma_mode(&mut self, mode: DmaMode) -> &mut Self {
        self.0.set_bit(1, mode.into());
        self
    }

    pub fn read_write_mode(&self) -> ReadWriteMode {
        self.0.get_bits(2..=3).try_into().unwrap()
    }

    pub fn with_read_write_mode(&mut self, mode: ReadWriteMode) -> &mut Self {
        self.0.set_bits(2..=3, mode.into());
        self
    }

    pub fn transfer_length(&self) -> u8 {
        self.0.get_bits(4..=5).try_into().unwrap()
    }

    pub fn with_transfer_length(&mut self, length: u8) -> &mut Self {
        debug_assert!(
            length < 5 && length > 0,
            "DMA length must be less then 5 and greater then 0"
        );
        self.0.set_bits(4..=5, (length - 1).try_into().unwrap());
        self
    }
}

pub const EXI_CHANNEL_0_IMM_DATA: VolAddress<u32, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x10) };

pub const EXI_CHANNEL_1_IMM_DATA: VolAddress<u32, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x24) };

pub const EXI_CHANNEL_2_IMM_DATA: VolAddress<u32, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x38) };
