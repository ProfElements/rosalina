use bit_field::BitField;
use voladdress::{Safe, VolAddress};

use super::{pi::InterruptState, vi::Enabled};

pub const BASE: usize = 0xCC00_0000;

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct CommandStatus(u16);

pub const COMMAND_STATUS: VolAddress<CommandStatus, Safe, Safe> = unsafe { VolAddress::new(BASE) };

impl CommandStatus {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        COMMAND_STATUS.read()
    }

    pub fn write(self) {
        COMMAND_STATUS.write(self);
    }

    pub fn fifo_overflow(&self) -> bool {
        self.0.get_bit(0)
    }

    pub fn with_fifo_overflow(&mut self, overflow: bool) -> &mut Self {
        self.0.set_bit(0, overflow);
        self
    }

    pub fn fifo_underflow(&self) -> bool {
        self.0.get_bit(1)
    }

    pub fn with_fifo_underflow(&mut self, underflow: bool) -> &mut Self {
        self.0.set_bit(1, underflow);
        self
    }

    pub fn fifo_read_idle(&self) -> bool {
        self.0.get_bit(2)
    }

    pub fn with_read_idle(&mut self, idle: bool) -> &mut Self {
        self.0.set_bit(2, idle);
        self
    }

    pub fn fifo_command_idle(&self) -> bool {
        self.0.get_bit(3)
    }

    pub fn with_command_idle(&mut self, idle: bool) -> &mut Self {
        self.0.set_bit(3, idle);
        self
    }

    pub fn fifo_breakpoint_interrupt(&self) -> InterruptState {
        self.0.get_bit(4).into()
    }

    pub fn with_fifo_breakpoint_interrupt(&mut self, state: InterruptState) -> &mut Self {
        self.0.set_bit(4, state.into());
        self
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct CommandControl(u16);

pub const COMMAND_CONTROL: VolAddress<CommandControl, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x2) };

impl CommandControl {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        COMMAND_CONTROL.read()
    }

    pub fn write(self) {
        COMMAND_CONTROL.write(self);
    }

    pub fn gp_fifo_read_enable(&self) -> Enabled {
        self.0.get_bit(0).into()
    }

    pub fn with_gp_fifo_read_enable(&mut self, enable: Enabled) -> &mut Self {
        self.0.set_bit(0, enable.into());
        self
    }

    pub fn command_processor_interrupt(&self) -> InterruptState {
        self.0.get_bit(1).into()
    }

    pub fn with_command_processor_interrupt(&mut self, state: InterruptState) -> &mut Self {
        self.0.set_bit(1, state.into());
        self
    }

    pub fn fifo_overflow_interrupt(&self) -> InterruptState {
        self.0.get_bit(2).into()
    }

    pub fn with_fifo_overflow_interrupt(&mut self, state: InterruptState) -> &mut Self {
        self.0.set_bit(2, state.into());
        self
    }

    pub fn fifo_underflow_interrupt(&self) -> InterruptState {
        self.0.get_bit(3).into()
    }

    pub fn with_fifo_underflow_interrupt(&mut self, state: InterruptState) -> &mut Self {
        self.0.set_bit(3, state.into());
        self
    }

    pub fn fifo_link_enable(&self) -> Enabled {
        self.0.get_bit(4).into()
    }

    pub fn with_fifo_link_enable(&mut self, enable: Enabled) -> &mut Self {
        self.0.set_bit(4, enable.into());
        self
    }

    pub fn fifo_breakpoint_interrupt(&self) -> InterruptState {
        self.0.get_bit(5).into()
    }

    pub fn with_fifo_breakpoint_interrupt(&mut self, state: InterruptState) -> &mut Self {
        self.0.set_bit(5, state.into());
        self
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct CommandClear(u16);

pub const COMMAND_CLEAR: VolAddress<CommandClear, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x4) };

impl CommandClear {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        COMMAND_CLEAR.read()
    }

    pub fn write(self) {
        COMMAND_CLEAR.write(self);
    }

    pub fn fifo_overflow(&self) -> bool {
        self.0.get_bit(0)
    }

    pub fn with_fifo_overflow(&mut self, overflow: bool) -> &mut Self {
        self.0.set_bit(0, overflow);
        self
    }

    pub fn fifo_underflow(&self) -> bool {
        self.0.get_bit(1)
    }

    pub fn with_fifo_underflow(&mut self, underflow: bool) -> &mut Self {
        self.0.set_bit(1, underflow);
        self
    }
}

pub const CP_FIFO_BASE_LO: VolAddress<u16, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x20) };
pub const CP_FIFO_BASE_HI: VolAddress<u16, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x22) };

pub const CP_FIFO_END_LO: VolAddress<u16, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x24) };
pub const CP_FIFO_END_HI: VolAddress<u16, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x26) };

pub const CP_FIFO_HIGH_MARK_LO: VolAddress<u16, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x28) };
pub const CP_FIFO_HIGH_MARK_HI: VolAddress<u16, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x2A) };

pub const CP_FIFO_LO_MARK_LO: VolAddress<u16, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x2C) };
pub const CP_FIFO_LO_MARK_HI: VolAddress<u16, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x2E) };

pub const CP_FIFO_READ_WRITE_DST_LO: VolAddress<u16, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x30) };
pub const CP_FIFO_READ_WRITE_DST_HI: VolAddress<u16, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x32) };

pub const CP_FIFO_WRITE_PTR_LO: VolAddress<u16, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x34) };
pub const CP_FIFO_WRITE_PTR_HI: VolAddress<u16, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x36) };

pub const CP_FIFO_READ_PTR_LO: VolAddress<u16, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x38) };
pub const CP_FIFO_READ_PTR_HI: VolAddress<u16, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x3A) };

pub const CP_FIFO_BP_PTR_LO: VolAddress<u16, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x3C) };
pub const CP_FIFO_BP_PTR_HI: VolAddress<u16, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x3E) };
