use voladdress::{Safe, VolAddress};

pub const BASE: usize = 0xCD006400;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct SiOutputBuf(u32);

pub const SI_CHANNEL_0_OUTPUT_BUF: VolAddress<SiOutputBuf, Safe, Safe> =
    unsafe { VolAddress::new(BASE) };

pub const SI_CHANNEL_1_OUTPUT_BUF: VolAddress<SiOutputBuf, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0xC) };

pub const SI_CHANNEL_2_OUTPUT_BUF: VolAddress<SiOutputBuf, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x18) };

pub const SI_CHANNEL_3_OUTPUT_BUF: VolAddress<SiOutputBuf, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x24) };

impl From<u32> for SiOutputBuf {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<SiOutputBuf> for u32 {
    fn from(value: SiOutputBuf) -> Self {
        value.0
    }
}

impl SiOutputBuf {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_zero() -> Self {
        SI_CHANNEL_0_OUTPUT_BUF.read()
    }

    pub fn read_one() -> Self {
        SI_CHANNEL_1_OUTPUT_BUF.read()
    }

    pub fn read_two() -> Self {
        SI_CHANNEL_2_OUTPUT_BUF.read()
    }

    pub fn read_three() -> Self {
        SI_CHANNEL_3_OUTPUT_BUF.read()
    }

    pub fn write_zero(self) {
        SI_CHANNEL_0_OUTPUT_BUF.write(self)
    }

    pub fn write_one(self) {
        SI_CHANNEL_1_OUTPUT_BUF.write(self)
    }

    pub fn write_two(self) {
        SI_CHANNEL_2_OUTPUT_BUF.write(self)
    }

    pub fn write_three(self) {
        SI_CHANNEL_3_OUTPUT_BUF.write(self)
    }
}
