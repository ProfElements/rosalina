use bit_field::BitField;
use voladdress::{Safe, VolAddress};

pub const BASE: usize = 0xCC00_3000;

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct InterruptCause(u32);

pub const INTERRUPT_CAUSE: VolAddress<InterruptCause, Safe, ()> = unsafe { VolAddress::new(BASE) };

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum InterruptState {
    Happened,
    Idle,
}

impl From<bool> for InterruptState {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Happened,
            false => Self::Idle,
        }
    }
}

impl From<InterruptState> for bool {
    fn from(value: InterruptState) -> Self {
        match value {
            InterruptState::Happened => true,
            InterruptState::Idle => false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ResetSwitchState {
    Pressed,
    Idle,
}

impl From<bool> for ResetSwitchState {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Idle,
            false => Self::Pressed,
        }
    }
}

impl From<ResetSwitchState> for bool {
    fn from(value: ResetSwitchState) -> Self {
        match value {
            ResetSwitchState::Pressed => false,
            ResetSwitchState::Idle => true,
        }
    }
}

impl InterruptCause {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        INTERRUPT_CAUSE.read()
    }

    pub fn gp_runtime_error(&self) -> InterruptState {
        self.0.get_bit(0).into()
    }

    pub fn reset_switch(&self) -> InterruptState {
        self.0.get_bit(1).into()
    }

    pub fn dvd_interface(&self) -> InterruptState {
        self.0.get_bit(2).into()
    }

    pub fn serial_interface(&self) -> InterruptState {
        self.0.get_bit(3).into()
    }

    pub fn external_interface(&self) -> InterruptState {
        self.0.get_bit(4).into()
    }

    pub fn audio_interface(&self) -> InterruptState {
        self.0.get_bit(5).into()
    }

    pub fn dsp_interface(&self) -> InterruptState {
        self.0.get_bit(6).into()
    }

    pub fn memory_interface(&self) -> InterruptState {
        self.0.get_bit(7).into()
    }

    pub fn video_interface(&self) -> InterruptState {
        self.0.get_bit(8).into()
    }

    pub fn pixel_engine_token(&self) -> InterruptState {
        self.0.get_bit(9).into()
    }

    pub fn pixel_engine_finish(&self) -> InterruptState {
        self.0.get_bit(10).into()
    }

    pub fn command_fifo(&self) -> InterruptState {
        self.0.get_bit(11).into()
    }

    pub fn debug(&self) -> InterruptState {
        self.0.get_bit(12).into()
    }

    pub fn high_speed_port(&self) -> InterruptState {
        self.0.get_bit(13).into()
    }

    pub fn interprocess_control(&self) -> InterruptState {
        self.0.get_bit(14).into()
    }

    pub fn reset_state(&self) -> ResetSwitchState {
        self.0.get_bit(15).into()
    }
}

impl From<u32> for InterruptCause {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<InterruptCause> for u32 {
    fn from(value: InterruptCause) -> Self {
        value.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct InterruptMask(u32);

pub const INTERRUPT_MASK: VolAddress<InterruptMask, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x4) };

pub enum Mask {
    Enabled,
    Disabled,
}

impl From<bool> for Mask {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Enabled,
            false => Self::Disabled,
        }
    }
}

impl From<Mask> for bool {
    fn from(value: Mask) -> Self {
        match value {
            Mask::Enabled => true,
            Mask::Disabled => false,
        }
    }
}

impl InterruptMask {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        INTERRUPT_MASK.read()
    }

    pub fn write(self) {
        INTERRUPT_MASK.write(self)
    }

    pub fn gp_runtime_error(&self) -> Mask {
        self.0.get_bit(0).into()
    }

    pub fn with_gp_runtime_error(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(0, mask.into());
        self
    }

    pub fn reset_switch(&self) -> Mask {
        self.0.get_bit(1).into()
    }

    pub fn with_reset_switch(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(1, mask.into());
        self
    }

    pub fn dvd_interface(&self) -> Mask {
        self.0.get_bit(2).into()
    }

    pub fn with_dvd_interface(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(2, mask.into());
        self
    }

    pub fn serial_interface(&self) -> Mask {
        self.0.get_bit(3).into()
    }

    pub fn with_serial_interface(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(3, mask.into());
        self
    }

    pub fn external_interface(&self) -> Mask {
        self.0.get_bit(4).into()
    }

    pub fn with_external_interface(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(4, mask.into());
        self
    }

    pub fn audio_interface(&self) -> Mask {
        self.0.get_bit(5).into()
    }

    pub fn with_audio_interface(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(5, mask.into());
        self
    }

    pub fn dsp_interface(&self) -> Mask {
        self.0.get_bit(6).into()
    }

    pub fn with_dsp_interface(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(6, mask.into());
        self
    }

    pub fn memory_interface(&self) -> Mask {
        self.0.get_bit(7).into()
    }

    pub fn with_memory_interface(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(7, mask.into());
        self
    }
    pub fn video_interface(&self) -> Mask {
        self.0.get_bit(8).into()
    }

    pub fn with_video_interface(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(8, mask.into());
        self
    }

    pub fn pixel_engine_token(&self) -> Mask {
        self.0.get_bit(9).into()
    }

    pub fn with_pixel_engine_token(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(9, mask.into());
        self
    }

    pub fn pixel_engine_finish(&self) -> Mask {
        self.0.get_bit(10).into()
    }

    pub fn with_pixel_engine_finish(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(10, mask.into());
        self
    }

    pub fn command_fifo(&self) -> Mask {
        self.0.get_bit(11).into()
    }

    pub fn with_command_fifo(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(11, mask.into());
        self
    }

    pub fn debug(&self) -> Mask {
        self.0.get_bit(12).into()
    }

    pub fn with_debug(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(12, mask.into());
        self
    }

    pub fn high_speed_port(&self) -> Mask {
        self.0.get_bit(13).into()
    }

    pub fn with_high_speed_port(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(13, mask.into());
        self
    }

    pub fn interprocess_control(&self) -> Mask {
        self.0.get_bit(14).into()
    }

    pub fn with_interprocess_control(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(14, mask.into());
        self
    }
}

impl From<u32> for InterruptMask {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<InterruptMask> for u32 {
    fn from(value: InterruptMask) -> Self {
        value.0
    }
}

// TODO: Move from usize for FIFO_BASE, FIFO_END, FIFO_WRITE_PTR to PhysAddr

pub const FIFO_BASE: VolAddress<usize, Safe, Safe> = unsafe { VolAddress::new(BASE + 0xC) };

pub const FIFO_END: VolAddress<usize, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x10) };

pub const FIFO_WRITE_PTR: VolAddress<usize, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x14) };

// TODO: Properly do FIFO_RESET https://github.com/dolphin-emu/dolphin/blob/master/Source/Core/Core/HW/ProcessorInterface.h
//pub const FIFO_RESET: VolAddress<usize, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x18) };

// TODO: Properly do RESET_CODE https://github.com/dolphin-emu/dolphin/blob/master/Source/Core/Core/HW/ProcessorInterface.h
//pub const RESET_CODE: VolAddress<usize, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x24) };

// TODO: Properly do FLIPPER_REV https://github.com/dolphin-emu/dolphin/blob/master/Source/Core/Core/HW/ProcessorInterface.h
//pub const FLIPPER_REV: VolAddress<usize, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x2C) };

// TODO: Properly do FLIPPER_UNK https://github.com/dolphin-emu/dolphin/blob/master/Source/Core/Core/HW/ProcessorInterface.h
//pub const FIFO_UNK: VolAddress<usize, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x30) };
