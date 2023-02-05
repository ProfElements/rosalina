use bit_field::BitField;
use voladdress::{Safe, VolAddress};

use super::pi::{InterruptState, Mask};

pub const BASE: usize = 0xCD006C00;

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AudioControl(u32);

pub const AUDIO_CTRL: VolAddress<AudioControl, Safe, Safe> = unsafe { VolAddress::new(BASE) };

impl From<u32> for AudioControl {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<AudioControl> for u32 {
    fn from(value: AudioControl) -> Self {
        value.0
    }
}

impl AudioControl {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        AUDIO_CTRL.read()
    }

    pub fn write(self) {
        AUDIO_CTRL.write(self);
    }

    pub fn playing_status(&self) -> PlayingStatus {
        self.0.get_bit(0).into()
    }

    pub fn with_playing_status(&mut self, status: PlayingStatus) -> &mut Self {
        self.0.set_bit(0, status.into());
        self
    }

    pub fn streaming_sample_rate(&self) -> SampleRate {
        self.0.get_bit(1).into()
    }

    pub fn with_streaming_sample_rate(&mut self, rate: SampleRate) -> &mut Self {
        self.0.set_bit(1, rate.into());
        self
    }

    pub fn audio_interrupt_mask(&self) -> Mask {
        self.0.get_bit(2).into()
    }

    pub fn with_audio_interrupt_mask(&mut self, mask: Mask) -> &mut Self {
        self.0.set_bit(2, mask.into());
        self
    }

    pub fn audio_interrupt(&self) -> InterruptState {
        self.0.get_bit(3).into()
    }

    pub fn with_audio_interrupt(&mut self, state: InterruptState) -> &mut Self {
        self.0.set_bit(3, state.into());
        self
    }

    pub fn interrupt_valid(&self) -> Valid {
        self.0.get_bit(4).into()
    }

    pub fn with_interrupt_valid(&mut self, valid: Valid) -> &mut Self {
        self.0.set_bit(4, valid.into());
        self
    }

    pub fn sample_count_clear(&self) -> Clear {
        self.0.get_bit(5).into()
    }

    pub fn with_sample_count_clear(&mut self, clear: Clear) -> &mut Self {
        self.0.set_bit(5, clear.into());
        self
    }

    pub fn dma_sample_rate(&self) -> SampleRate {
        self.0.get_bit(6).into()
    }

    pub fn with_dma_sample_rate(&mut self, rate: SampleRate) -> &mut Self {
        self.0.set_bit(7, rate.into());
        self
    }
}

pub enum PlayingStatus {
    Playing,
    Idle,
}

impl From<bool> for PlayingStatus {
    fn from(value: bool) -> Self {
        if value {
            Self::Playing
        } else {
            Self::Idle
        }
    }
}

impl From<PlayingStatus> for bool {
    fn from(value: PlayingStatus) -> Self {
        match value {
            PlayingStatus::Playing => true,
            PlayingStatus::Idle => false,
        }
    }
}

pub enum SampleRate {
    FortyEightKhz,
    ThirtyTwoKhz,
}

impl From<bool> for SampleRate {
    fn from(value: bool) -> Self {
        if value {
            Self::ThirtyTwoKhz
        } else {
            Self::FortyEightKhz
        }
    }
}

impl From<SampleRate> for bool {
    fn from(value: SampleRate) -> Self {
        match value {
            SampleRate::ThirtyTwoKhz => true,
            SampleRate::FortyEightKhz => false,
        }
    }
}

pub enum Valid {
    Valid,
    Invalid,
}

impl From<bool> for Valid {
    fn from(value: bool) -> Self {
        if value {
            Self::Invalid
        } else {
            Self::Valid
        }
    }
}

impl From<Valid> for bool {
    fn from(value: Valid) -> Self {
        match value {
            Valid::Valid => false,
            Valid::Invalid => true,
        }
    }
}

pub enum Clear {
    Clear,
    Idle,
}

impl From<bool> for Clear {
    fn from(value: bool) -> Self {
        if value {
            Self::Clear
        } else {
            Self::Idle
        }
    }
}

impl From<Clear> for bool {
    fn from(value: Clear) -> Self {
        match value {
            Clear::Clear => true,
            Clear::Idle => false,
        }
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AudioVolume(u32);

pub const AUDIO_VOLUME: VolAddress<AudioVolume, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x4) };

impl From<u32> for AudioVolume {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<AudioVolume> for u32 {
    fn from(value: AudioVolume) -> Self {
        value.0
    }
}

impl AudioVolume {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        AUDIO_VOLUME.read()
    }

    pub fn write(self) {
        AUDIO_VOLUME.write(self);
    }

    pub fn right(&self) -> u8 {
        self.0.get_bits(0..=7).try_into().unwrap()
    }

    pub fn with_right(&mut self, vol: u8) -> &mut Self {
        self.0.set_bits(0..=7, vol.into());
        self
    }

    pub fn left(&self) -> u8 {
        self.0.get_bits(8..=15).try_into().unwrap()
    }

    pub fn with_left(&mut self, vol: u8) -> &mut Self {
        self.0.set_bits(8..=15, vol.into());
        self
    }

    pub fn with_volume(&mut self, left_vol: u8, right_vol: u8) -> &mut Self {
        self.with_left(left_vol).with_right(right_vol)
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AudioSamples(u32);

pub const AUDIO_SAMPLES: VolAddress<AudioSamples, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x8) };

impl From<u32> for AudioSamples {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<AudioSamples> for u32 {
    fn from(value: AudioSamples) -> Self {
        value.0
    }
}

impl AudioSamples {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        AUDIO_SAMPLES.read()
    }

    pub fn samples(&self) -> u32 {
        self.0.get_bits(0..=31)
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SampleInterruptCount(u32);

pub const SAMPLE_INTERRRUPT_COUNT: VolAddress<SampleInterruptCount, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0xC) };

impl From<u32> for SampleInterruptCount {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<SampleInterruptCount> for u32 {
    fn from(value: SampleInterruptCount) -> Self {
        value.0
    }
}

impl SampleInterruptCount {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        SAMPLE_INTERRRUPT_COUNT.read()
    }

    pub fn write(self) {
        SAMPLE_INTERRRUPT_COUNT.write(self);
    }

    pub fn count(&self) -> u32 {
        self.0.get_bits(0..=31)
    }

    pub fn with_count(&mut self, count: u32) -> &mut Self {
        self.0.set_bits(0..=31, count);
        self
    }
}
