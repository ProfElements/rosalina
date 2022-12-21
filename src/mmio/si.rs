use bit_field::BitField;
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
        SI_CHANNEL_0_OUTPUT_BUF.write(self);
    }

    pub fn write_one(self) {
        SI_CHANNEL_1_OUTPUT_BUF.write(self);
    }

    pub fn write_two(self) {
        SI_CHANNEL_2_OUTPUT_BUF.write(self);
    }

    pub fn write_three(self) {
        SI_CHANNEL_3_OUTPUT_BUF.write(self);
    }

    pub fn output_one(&self) -> u8 {
        self.0.get_bits(0..=7).try_into().unwrap()
    }

    pub fn with_output_one(&mut self, output: u8) -> &mut Self {
        self.0.set_bits(0..=7, output.into());
        self
    }

    pub fn output_zero(&self) -> u8 {
        self.0.get_bits(8..=15).try_into().unwrap()
    }

    pub fn with_output_zero(&mut self, output: u8) -> &mut Self {
        self.0.set_bits(8..=15, output.into());
        self
    }

    pub fn cmd(&self) -> u8 {
        self.0.get_bits(16..=23).try_into().unwrap()
    }

    pub fn with_cmd(&mut self, cmd: u8) -> &mut Self {
        self.0.set_bits(16..=23, cmd.into());
        self
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct SiInputBufHi(u32);

pub const SI_CHANNEL_0_INPUT_BUF_HI: VolAddress<SiInputBufHi, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x4) };

pub const SI_CHANNEL_1_INPUT_BUF_HI: VolAddress<SiInputBufHi, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x10) };

pub const SI_CHANNEL_2_INPUT_BUF_HI: VolAddress<SiInputBufHi, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x1C) };

pub const SI_CHANNEL_3_INPUT_BUF_HI: VolAddress<SiInputBufHi, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x28) };

impl From<u32> for SiInputBufHi {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<SiInputBufHi> for u32 {
    fn from(value: SiInputBufHi) -> Self {
        value.0
    }
}

impl SiInputBufHi {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_zero() -> Self {
        SI_CHANNEL_0_INPUT_BUF_HI.read()
    }

    pub fn read_one() -> Self {
        SI_CHANNEL_1_INPUT_BUF_HI.read()
    }

    pub fn read_two() -> Self {
        SI_CHANNEL_1_INPUT_BUF_HI.read()
    }

    pub fn read_three() -> Self {
        SI_CHANNEL_3_INPUT_BUF_HI.read()
    }

    pub fn error_status(&self) -> ErrorStatus {
        self.0.get_bit(31).into()
    }

    pub fn error_latch(&self) -> ErrorLatch {
        self.0.get_bit(30).into()
    }

    pub fn byte_zero(&self) -> u8 {
        self.0.get_bits(24..=29).try_into().unwrap()
    }

    pub fn byte_one(&self) -> u8 {
        self.0.get_bits(16..=23).try_into().unwrap()
    }

    pub fn byte_two(&self) -> u8 {
        self.0.get_bits(8..=15).try_into().unwrap()
    }

    pub fn byte_three(&self) -> u8 {
        self.0.get_bits(0..=7).try_into().unwrap()
    }
}

pub enum ErrorStatus {
    Happened,
    Idle,
}

impl From<bool> for ErrorStatus {
    fn from(value: bool) -> Self {
        if value {
            Self::Happened
        } else {
            Self::Idle
        }
    }
}

impl From<ErrorStatus> for bool {
    fn from(value: ErrorStatus) -> Self {
        match value {
            ErrorStatus::Happened => true,
            ErrorStatus::Idle => false,
        }
    }
}

pub enum ErrorLatch {
    Latched,
    Idle,
}

impl From<bool> for ErrorLatch {
    fn from(value: bool) -> Self {
        if value {
            Self::Latched
        } else {
            Self::Idle
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct SiInputBufLo(u32);

pub const SI_CHANNEL_0_INPUT_BUF_LO: VolAddress<SiInputBufLo, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x8) };

pub const SI_CHANNEL_1_INPUT_BUF_LO: VolAddress<SiInputBufLo, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x14) };

pub const SI_CHANNEL_2_INPUT_BUF_LO: VolAddress<SiInputBufLo, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x20) };

pub const SI_CHANNEL_3_INPUT_BUF_LO: VolAddress<SiInputBufLo, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x2C) };

impl From<u32> for SiInputBufLo {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<SiInputBufLo> for u32 {
    fn from(value: SiInputBufLo) -> Self {
        value.0
    }
}

impl SiInputBufLo {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_zero() -> Self {
        SI_CHANNEL_0_INPUT_BUF_LO.read()
    }

    pub fn read_one() -> Self {
        SI_CHANNEL_1_INPUT_BUF_LO.read()
    }

    pub fn read_two() -> Self {
        SI_CHANNEL_1_INPUT_BUF_LO.read()
    }

    pub fn read_three() -> Self {
        SI_CHANNEL_3_INPUT_BUF_LO.read()
    }

    pub fn byte_four(&self) -> u8 {
        self.0.get_bits(24..=31).try_into().unwrap()
    }

    pub fn byte_five(&self) -> u8 {
        self.0.get_bits(16..=23).try_into().unwrap()
    }

    pub fn byte_six(&self) -> u8 {
        self.0.get_bits(8..=15).try_into().unwrap()
    }

    pub fn byte_seven(&self) -> u8 {
        self.0.get_bits(0..=7).try_into().unwrap()
    }
}
