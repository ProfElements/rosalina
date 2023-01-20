use bit_field::BitField;
use voladdress::{Safe, VolAddress};

use super::pi::InterruptState;

pub const BASE: usize = 0xCD00_0000;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct IpcRequestAddr(usize);

pub const PPC_REQUEST: VolAddress<IpcRequestAddr, Safe, Safe> = unsafe { VolAddress::new(BASE) };

pub const ARM_REQUEST: VolAddress<IpcRequestAddr, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x08) };

impl From<usize> for IpcRequestAddr {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<IpcRequestAddr> for usize {
    fn from(value: IpcRequestAddr) -> Self {
        value.0
    }
}

impl IpcRequestAddr {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_ppc() -> Self {
        PPC_REQUEST.read()
    }

    pub fn read_arm() -> Self {
        ARM_REQUEST.read()
    }

    pub fn write_ppc(self) {
        PPC_REQUEST.write(self);
    }

    pub fn addr(&self) -> usize {
        self.0.get_bits(0..=31).try_into().unwrap()
    }

    pub fn with_addr(&mut self, addr: usize) -> &mut Self {
        self.0.set_bits(0..=31, addr);
        self
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct IpcControl(u32);

pub const PPC_CONTROL: VolAddress<IpcControl, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x4) };
pub const ARM_CONTROL: VolAddress<IpcControl, Safe, Safe> = unsafe { VolAddress::new(BASE + 0xC) };

impl From<u32> for IpcControl {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<IpcControl> for u32 {
    fn from(value: IpcControl) -> Self {
        value.0
    }
}

impl IpcControl {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_ppc() -> Self {
        PPC_CONTROL.read()
    }

    pub fn read_arm() -> Self {
        ARM_CONTROL.read()
    }

    pub fn write_ppc(self) {
        PPC_CONTROL.write(self);
    }

    pub fn write_arm(self) {
        ARM_CONTROL.write(self);
    }

    pub fn x1(&self) -> bool {
        self.0.get_bit(0)
    }

    pub fn with_x1(&mut self, x1: bool) -> &mut Self {
        self.0.set_bit(0, x1);
        self
    }

    pub fn x2(&self) -> bool {
        self.0.get_bit(1)
    }

    pub fn with_x2(&mut self, x2: bool) -> &mut Self {
        self.0.set_bit(1, x2);
        self
    }

    pub fn y1(&self) -> bool {
        self.0.get_bit(2)
    }

    pub fn with_y1(&mut self, y1: bool) -> &mut Self {
        self.0.set_bit(2, y1);
        self
    }

    pub fn y2(&self) -> bool {
        self.0.get_bit(3)
    }

    pub fn with_y2(&mut self, y2: bool) -> &mut Self {
        self.0.set_bit(3, y2);
        self
    }

    pub fn ix1(&self) -> bool {
        self.0.get_bit(4)
    }

    pub fn with_ix1(&mut self, ix1: bool) -> &mut Self {
        self.0.set_bit(4, ix1);
        self
    }

    pub fn ix2(&self) -> bool {
        self.0.get_bit(5)
    }

    pub fn with_ix2(&mut self, ix2: bool) -> &mut Self {
        self.0.set_bit(5, ix2);
        self
    }

    pub fn iy1(&self) -> bool {
        self.0.get_bit(6)
    }

    pub fn with_iy1(&mut self, iy1: bool) -> &mut Self {
        self.0.set_bit(6, iy1);
        self
    }

    pub fn iy2(&self) -> bool {
        self.0.get_bit(7)
    }

    pub fn with_iy2(&mut self, iy2: bool) -> &mut Self {
        self.0.set_bit(7, iy2);
        self
    }
}

pub const PPC_SPEED: VolAddress<u32, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x18) };

pub const VI_SOLID: VolAddress<u32, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x24) };

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct IpcInterruptFlags(u32);

pub const PPC_FLAGS: VolAddress<IpcInterruptFlags, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x30) };

pub const ARM_FLAGS: VolAddress<IpcInterruptFlags, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x38) };

impl From<u32> for IpcInterruptFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<IpcInterruptFlags> for u32 {
    fn from(value: IpcInterruptFlags) -> Self {
        value.0
    }
}

impl IpcInterruptFlags {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_ppc() -> Self {
        PPC_FLAGS.read()
    }

    pub fn read_arm() -> Self {
        ARM_FLAGS.read()
    }

    pub fn write_ppc(self) {
        PPC_FLAGS.write(self);
    }

    pub fn write_arm(self) {
        ARM_FLAGS.write(self);
    }

    pub fn ipc_interrupt(&self) -> InterruptState {
        self.0.get_bit(30).into()
    }

    pub fn with_ipc_interupt(&mut self, interrupt: InterruptState) -> &mut Self {
        self.0.set_bit(30, interrupt.into());
        self
    }
}
