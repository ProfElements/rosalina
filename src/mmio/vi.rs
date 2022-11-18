use bit_field::BitField;
use voladdress::{Safe, VolAddress};

use super::pi::InterruptState;

pub const BASE: usize = 0xCC00_2000;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct VerticalTiming(u16);

pub const VERTICAL_TIMING: VolAddress<VerticalTiming, Safe, Safe> =
    unsafe { VolAddress::new(BASE) };

impl From<u16> for VerticalTiming {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<VerticalTiming> for u16 {
    fn from(value: VerticalTiming) -> Self {
        value.0
    }
}

impl VerticalTiming {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        VERTICAL_TIMING.read()
    }

    pub fn write(self) {
        VERTICAL_TIMING.write(self);
    }

    pub fn active_video_lines(&self) -> u16 {
        self.0.get_bits(4..=13)
    }

    pub fn with_active_video_lines(&mut self, active_video_lines: u16) -> &mut Self {
        self.0.set_bits(4..=13, active_video_lines);
        self
    }

    pub fn equalization_pulse(&self) -> u8 {
        self.0.get_bits(0..=3).try_into().unwrap()
    }

    pub fn with_equalizaion_pulse(&mut self, equalization_pulse: u8) -> &mut Self {
        self.0.set_bits(0..=3, equalization_pulse.into());
        self
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct DisplayConfig(u16);

pub const DISPLAY_CONFIG: VolAddress<DisplayConfig, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x2) };

impl From<u16> for DisplayConfig {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<DisplayConfig> for u16 {
    fn from(value: DisplayConfig) -> Self {
        value.0
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u16)]
pub enum VideoFormat {
    Ntsc,
    Pal,
    Mpal,
    Debug,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct InvalidVideoFormatError;

impl TryFrom<u16> for VideoFormat {
    type Error = InvalidVideoFormatError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Ntsc),
            1 => Ok(Self::Pal),
            2 => Ok(Self::Mpal),
            3 => Ok(Self::Debug),
            _ => Err(InvalidVideoFormatError),
        }
    }
}

impl From<VideoFormat> for u16 {
    fn from(value: VideoFormat) -> Self {
        match value {
            VideoFormat::Ntsc => 0,
            VideoFormat::Pal => 1,
            VideoFormat::Mpal => 2,
            VideoFormat::Debug => 3,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum DisplayLatchState {
    Off,
    On,
    OnForFieldOne,
    OnForFieldTwo,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct InvalidDisplayLatchError;

impl TryFrom<u16> for DisplayLatchState {
    type Error = InvalidDisplayLatchError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Off),
            1 => Ok(Self::OnForFieldOne),
            2 => Ok(Self::OnForFieldTwo),
            3 => Ok(Self::On),
            _ => Err(InvalidDisplayLatchError),
        }
    }
}

impl From<DisplayLatchState> for u16 {
    fn from(value: DisplayLatchState) -> Self {
        match value {
            DisplayLatchState::Off => 0,
            DisplayLatchState::OnForFieldOne => 1,
            DisplayLatchState::OnForFieldTwo => 2,
            DisplayLatchState::On => 3,
        }
    }
}

pub enum Display3dMode {
    ThreeDimension,
    TwoDimension,
}

impl From<bool> for Display3dMode {
    fn from(value: bool) -> Self {
        match value {
            true => Self::ThreeDimension,
            false => Self::TwoDimension,
        }
    }
}

impl From<Display3dMode> for bool {
    fn from(value: Display3dMode) -> Self {
        match value {
            Display3dMode::ThreeDimension => true,
            Display3dMode::TwoDimension => false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum DisplayInterlacedMode {
    Interlaced,
    NonInterlaced,
}

impl From<bool> for DisplayInterlacedMode {
    fn from(value: bool) -> Self {
        match value {
            true => Self::NonInterlaced,
            false => Self::Interlaced,
        }
    }
}

impl From<DisplayInterlacedMode> for bool {
    fn from(value: DisplayInterlacedMode) -> Self {
        match value {
            DisplayInterlacedMode::Interlaced => false,
            DisplayInterlacedMode::NonInterlaced => true,
        }
    }
}

pub enum Reset {
    Reset,
    NoReset,
}

impl From<bool> for Reset {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Reset,
            false => Self::NoReset,
        }
    }
}

impl From<Reset> for bool {
    fn from(value: Reset) -> Self {
        match value {
            Reset::Reset => true,
            Reset::NoReset => false,
        }
    }
}

pub enum Enabled {
    Enabled,
    Disabled,
}

impl From<bool> for Enabled {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Enabled,
            false => Self::Disabled,
        }
    }
}

impl From<Enabled> for bool {
    fn from(value: Enabled) -> Self {
        match value {
            Enabled::Enabled => true,
            Enabled::Disabled => false,
        }
    }
}

impl DisplayConfig {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        DISPLAY_CONFIG.read()
    }

    pub fn write(self) {
        DISPLAY_CONFIG.write(self)
    }

    pub fn video_format(&self) -> VideoFormat {
        self.0.get_bits(8..=9).try_into().unwrap()
    }

    pub fn with_video_format(&mut self, format: VideoFormat) -> &mut Self {
        self.0.set_bits(8..=9, format.into());
        self
    }

    pub fn display_latch_one(&self) -> DisplayLatchState {
        self.0.get_bits(6..=7).try_into().unwrap()
    }

    pub fn with_display_latch_one(&mut self, latch: DisplayLatchState) -> &mut Self {
        self.0.set_bits(6..=7, latch.into());
        self
    }

    pub fn display_latch_two(&self) -> DisplayLatchState {
        self.0.get_bits(4..=5).try_into().unwrap()
    }

    pub fn with_display_latch_two(&mut self, latch: DisplayLatchState) -> &mut Self {
        self.0.set_bits(4..=5, latch.into());
        self
    }

    pub fn display_3d_mode(&self) -> Display3dMode {
        self.0.get_bit(3).into()
    }

    pub fn with_display_3d_mode(&mut self, mode: Display3dMode) -> &mut Self {
        self.0.set_bit(3, mode.into());
        self
    }

    pub fn display_interlaced_mode(&self) -> DisplayInterlacedMode {
        self.0.get_bit(2).into()
    }

    pub fn with_display_interlaced_mode(&mut self, mode: DisplayInterlacedMode) -> &mut Self {
        self.0.set_bit(2, mode.into());
        self
    }

    pub fn reset(&self) -> Reset {
        self.0.get_bit(1).into()
    }

    pub fn with_reset(&mut self, reset: Reset) -> &mut Self {
        self.0.set_bit(1, reset.into());
        self
    }

    pub fn enabled(&self) -> Enabled {
        self.0.get_bit(0).into()
    }

    pub fn with_enabled(&mut self, enable: Enabled) -> &mut Self {
        self.0.set_bit(0, enable.into());
        self
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct HorizontalTimingZero(u32);

pub const HORIZONTAL_TIMING_ZERO: VolAddress<HorizontalTimingZero, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x4) };

impl From<u32> for HorizontalTimingZero {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<HorizontalTimingZero> for u32 {
    fn from(value: HorizontalTimingZero) -> Self {
        value.0
    }
}

impl HorizontalTimingZero {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        HORIZONTAL_TIMING_ZERO.read()
    }

    pub fn write(self) {
        HORIZONTAL_TIMING_ZERO.write(self)
    }

    pub fn color_burst_start(&self) -> u8 {
        self.0.get_bits(24..=30).try_into().unwrap()
    }

    pub fn with_color_burst_start(&mut self, start: u8) -> &mut Self {
        debug_assert!(start < 2 ^ 7, "Color burst start must be less then 128");
        self.0.set_bits(24..=30, start.into());
        self
    }

    pub fn color_burst_end(&self) -> u8 {
        self.0.get_bits(16..=22).try_into().unwrap()
    }

    pub fn with_color_burst_end(&mut self, end: u8) -> &mut Self {
        debug_assert!(end < 2 ^ 7, "Color burst end must be less than 128");
        self.0.set_bits(16..=22, end.into());
        self
    }

    pub fn halfline_width(&self) -> u16 {
        self.0.get_bits(0..=9).try_into().unwrap()
    }

    pub fn with_halfline_width(&mut self, width: u16) -> &mut Self {
        debug_assert!(width < 2 ^ 10, "Halfline width must be less than 1024");
        self.0.set_bits(0..=9, width.into());
        self
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct HorizontalTimingOne(u32);

pub const HORIZONTAL_TIMING_ONE: VolAddress<HorizontalTimingOne, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x8) };

impl From<u32> for HorizontalTimingOne {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<HorizontalTimingOne> for u32 {
    fn from(value: HorizontalTimingOne) -> Self {
        value.0
    }
}

impl HorizontalTimingOne {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        HORIZONTAL_TIMING_ONE.read()
    }

    pub fn write(self) {
        HORIZONTAL_TIMING_ONE.write(self);
    }

    pub fn horizontal_blanking_start(&self) -> u16 {
        self.0.get_bits(17..=26).try_into().unwrap()
    }

    pub fn with_horizontal_blanking_start(&mut self, start: u16) -> &mut Self {
        debug_assert!(
            start < 2 ^ 10,
            "Horizontal blanking start must be less than 1024"
        );
        self.0.set_bits(17..=26, start.into());
        self
    }

    pub fn horizontal_blanking_end(&self) -> u16 {
        self.0.get_bits(7..=16).try_into().unwrap()
    }

    pub fn with_horizontal_blanking_end(&mut self, end: u16) -> &mut Self {
        debug_assert!(
            end < 2 ^ 10,
            "Horizontal blanking end must be less than 1024"
        );
        self.0.set_bits(7..=16, end.into());
        self
    }

    pub fn horizontal_sync_width(&self) -> u8 {
        self.0.get_bits(0..=6).try_into().unwrap()
    }

    pub fn with_horizontal_sync_width(&mut self, width: u16) -> &mut Self {
        debug_assert!(width < 2 ^ 7, "Horizontal sync with must be less then 128");
        self.0.set_bits(0..=6, width.into());
        self
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct FieldVerticalTiming(u32);

pub const ODD_FIELD_TIMING: VolAddress<FieldVerticalTiming, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0xC) };

pub const EVEN_FIELD_TIMING: VolAddress<FieldVerticalTiming, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x10) };

impl From<u32> for FieldVerticalTiming {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<FieldVerticalTiming> for u32 {
    fn from(value: FieldVerticalTiming) -> Self {
        value.0
    }
}

impl FieldVerticalTiming {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_odd() -> Self {
        ODD_FIELD_TIMING.read()
    }

    pub fn read_even() -> Self {
        EVEN_FIELD_TIMING.read()
    }

    pub fn write_odd(self) {
        ODD_FIELD_TIMING.write(self);
    }

    pub fn write_even(self) {
        EVEN_FIELD_TIMING.write(self);
    }

    pub fn pre_blanking(&self) -> u16 {
        self.0.get_bits(0..=9).try_into().unwrap()
    }

    pub fn with_pre_blanking(&mut self, blank: u16) -> &mut Self {
        debug_assert!(blank < 2 ^ 10, "Pre blanking must be less then 1024");
        self.0.set_bits(0..=9, blank.into());
        self
    }

    pub fn post_blanking(&self) -> u16 {
        self.0.get_bits(16..=25).try_into().unwrap()
    }

    pub fn with_post_blanking(&mut self, blank: u16) -> &mut Self {
        debug_assert!(blank < 2 ^ 10, "Post blanking must be less then 1024");
        self.0.set_bits(16..=25, blank.into());
        self
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct BurstBlankingInterval(u32);

pub const ODD_BURST_BLANKING_INTERVAL: VolAddress<BurstBlankingInterval, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x14) };

pub const EVEN_BURST_BLANKING_INTERVAL: VolAddress<BurstBlankingInterval, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x18) };

impl From<u32> for BurstBlankingInterval {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<BurstBlankingInterval> for u32 {
    fn from(value: BurstBlankingInterval) -> Self {
        value.0
    }
}

impl BurstBlankingInterval {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_odd() -> Self {
        ODD_BURST_BLANKING_INTERVAL.read()
    }

    pub fn read_even() -> Self {
        EVEN_BURST_BLANKING_INTERVAL.read()
    }

    pub fn write_odd(self) {
        ODD_BURST_BLANKING_INTERVAL.write(self)
    }

    pub fn write_even(self) {
        EVEN_BURST_BLANKING_INTERVAL.write(self)
    }

    pub fn burst_start(&self) -> u8 {
        self.0.get_bits(0..=4).try_into().unwrap()
    }

    pub fn with_burst_start(&mut self, start: u8) -> &mut Self {
        debug_assert!(start < 2 ^ 5, " Start must be less then 32");
        self.0.set_bits(0..=4, start.into());
        self
    }

    pub fn burst_end(&self) -> u16 {
        self.0.get_bits(5..=15).try_into().unwrap()
    }

    pub fn with_burst_end(&mut self, end: u16) -> &mut Self {
        debug_assert!(end < 2 ^ 11, "End must be less then 2048");
        self.0.set_bits(5..=15, end.into());
        self
    }

    pub fn burst_start_two(&self) -> u8 {
        self.0.get_bits(16..=20).try_into().unwrap()
    }

    pub fn with_burst_start_two(&mut self, start: u8) -> &mut Self {
        debug_assert!(start < 2 ^ 5, " Start must be less then 32");
        self.0.set_bits(16..=20, start.into());
        self
    }

    pub fn burst_end_two(&self) -> u16 {
        self.0.get_bits(21..=31).try_into().unwrap()
    }

    pub fn with_burst_end_two(&mut self, end: u16) -> &mut Self {
        debug_assert!(end < 2 ^ 11, "End must be less then 2048");
        self.0.set_bits(21..=31, end.into());
        self
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct Framebuffer(u32);

pub const FRAMEBUF_T_L: VolAddress<Framebuffer, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x1C) };

pub const FRAMEBUF_T_R: VolAddress<Framebuffer, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x20) };

pub const FRAMEBUF_B_L: VolAddress<Framebuffer, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x24) };

pub const FRAMEBUF_B_R: VolAddress<Framebuffer, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x28) };

impl From<u32> for Framebuffer {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Framebuffer> for u32 {
    fn from(value: Framebuffer) -> Self {
        value.0
    }
}

pub enum AddrOffset {
    Offset,
    None,
}

impl From<bool> for AddrOffset {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Offset,
            false => Self::None,
        }
    }
}

impl From<AddrOffset> for bool {
    fn from(value: AddrOffset) -> Self {
        match value {
            AddrOffset::Offset => true,
            AddrOffset::None => false,
        }
    }
}

impl Framebuffer {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_top_left() -> Self {
        FRAMEBUF_T_L.read()
    }

    pub fn write_top_left(self) {
        FRAMEBUF_T_L.write(self);
    }

    pub fn read_top_right() -> Self {
        FRAMEBUF_T_R.read()
    }

    pub fn write_top_right(self) {
        FRAMEBUF_T_R.write(self);
    }

    pub fn read_bottom_left() -> Self {
        FRAMEBUF_B_L.read()
    }

    pub fn write_bottom_left(self) {
        FRAMEBUF_B_L.write(self);
    }

    pub fn read_bottom_right() -> Self {
        FRAMEBUF_B_R.read()
    }

    pub fn write_bottom_right(self) {
        FRAMEBUF_B_R.write(self)
    }

    pub fn addr(&self) -> usize {
        self.0.get_bits(0..=23).try_into().unwrap()
    }

    pub fn with_addr(&mut self, addr: u32) -> &mut Self {
        if addr <= 0x00fffe00 {
            self.0.set_bits(0..=23, addr);
            self
        } else {
            self.0.set_bits(0..=23, addr >> 5);
            self.with_addr_offset(AddrOffset::Offset)
        }
    }

    pub fn horizontal_offset(&self) -> u8 {
        self.0.get_bits(24..=27).try_into().unwrap()
    }

    pub fn with_horizontal_offset(&mut self, offset: u8) -> &mut Self {
        debug_assert!(offset < 2 ^ 4, "Horizontal offset must be less then 16");
        self.0.set_bits(24..=27, offset.into());
        self
    }

    pub fn addr_offset(&self) -> AddrOffset {
        self.0.get_bit(28).into()
    }

    pub fn with_addr_offset(&mut self, offset: AddrOffset) -> &mut Self {
        self.0.set_bit(28, offset.into());
        self
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct VerticalPos(u16);

pub const VERTICAL_POS: VolAddress<VerticalPos, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x2C) };

impl From<u16> for VerticalPos {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<VerticalPos> for u16 {
    fn from(value: VerticalPos) -> Self {
        value.0
    }
}

impl VerticalPos {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        VERTICAL_POS.read()
    }

    pub fn pos(&self) -> u16 {
        self.0.get_bits(0..=9)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct HorizontalPos(u16);

pub const HORIZONTAL_POS: VolAddress<HorizontalPos, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x2E) };

impl HorizontalPos {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        HORIZONTAL_POS.read()
    }

    pub fn pos(&self) -> u16 {
        self.0.get_bits(0..=9)
    }
}

impl From<u16> for HorizontalPos {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<HorizontalPos> for u16 {
    fn from(value: HorizontalPos) -> Self {
        value.0
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct DisplayInterrupt(u32);

pub const DISP_INT_0: VolAddress<DisplayInterrupt, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x30) };

pub const DISP_INT_1: VolAddress<DisplayInterrupt, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x34) };

pub const DISP_INT_2: VolAddress<DisplayInterrupt, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x38) };

pub const DISP_INT_3: VolAddress<DisplayInterrupt, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x3C) };

impl From<u32> for DisplayInterrupt {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<DisplayInterrupt> for u32 {
    fn from(value: DisplayInterrupt) -> Self {
        value.0
    }
}

impl DisplayInterrupt {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_zero() -> Self {
        DISP_INT_0.read()
    }

    pub fn read_one() -> Self {
        DISP_INT_1.read()
    }

    pub fn read_two() -> Self {
        DISP_INT_2.read()
    }

    pub fn read_three() -> Self {
        DISP_INT_3.read()
    }

    pub fn write_zero(self) {
        DISP_INT_0.write(self)
    }

    pub fn write_one(self) {
        DISP_INT_1.write(self)
    }

    pub fn write_two(self) {
        DISP_INT_2.write(self)
    }

    pub fn write_three(self) {
        DISP_INT_3.write(self)
    }

    pub fn horizontal_pos(&self) -> u16 {
        self.0.get_bits(0..=9).try_into().unwrap()
    }

    pub fn with_horizontal_pos(&mut self, pos: u16) -> &mut Self {
        debug_assert!(pos < 2 ^ 10, "Horizontal position must be less then 1024");
        self.0.set_bits(0..=9, pos.into());
        self
    }

    pub fn vertical_pos(&self) -> u16 {
        self.0.get_bits(16..=25).try_into().unwrap()
    }

    pub fn with_vertical_pos(&mut self, pos: u16) -> &mut Self {
        debug_assert!(pos < 2 ^ 10, "Vertical position must be less then 1024");
        self.0.set_bits(16..=25, pos.into());
        self
    }

    pub fn enable(&self) -> Enabled {
        self.0.get_bit(28).into()
    }

    pub fn with_enable(&mut self, enable: Enabled) -> &mut Self {
        self.0.set_bit(28, enable.into());
        self
    }

    pub fn status(&self) -> InterruptState {
        self.0.get_bit(31).into()
    }

    pub fn with_status(&mut self, status: InterruptState) -> &mut Self {
        self.0.set_bit(31, status.into());
        self
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct DisplayLatch(u32);

pub const DISPLAY_LATCH_ZERO: VolAddress<DisplayLatch, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x40) };

pub const DISPLAY_LATCH_ONE: VolAddress<DisplayLatch, Safe, Safe> =
    unsafe { VolAddress::new(BASE + 0x44) };

impl From<u32> for DisplayLatch {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<DisplayLatch> for u32 {
    fn from(value: DisplayLatch) -> Self {
        value.0
    }
}

pub enum Trigger {
    Triggered,
    Idle,
}

impl From<bool> for Trigger {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Triggered,
            false => Self::Idle,
        }
    }
}

impl From<Trigger> for bool {
    fn from(value: Trigger) -> Self {
        match value {
            Trigger::Triggered => true,
            Trigger::Idle => false,
        }
    }
}

impl DisplayLatch {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read_zero() -> Self {
        DISPLAY_LATCH_ZERO.read()
    }

    pub fn read_one() -> Self {
        DISPLAY_LATCH_ONE.read()
    }

    pub fn write_zero(self) {
        DISPLAY_LATCH_ZERO.write(self);
    }

    pub fn write_one(self) {
        DISPLAY_LATCH_ONE.write(self);
    }

    pub fn horizontal_count(&self) -> u16 {
        self.0.get_bits(0..=10).try_into().unwrap()
    }

    pub fn with_horizontal_count(&mut self, count: u16) -> &mut Self {
        debug_assert!(count < 2 ^ 11, "Horizontal count must be less then 2048");
        self.0.set_bits(0..=10, count.into());
        self
    }

    pub fn vertical_count(&self) -> u16 {
        self.0.get_bits(16..=26).try_into().unwrap()
    }

    pub fn with_vertical_count(&mut self, count: u16) -> &mut Self {
        debug_assert!(count < 2 ^ 11, "Vertical count must be less then 2048");
        self.0.set_bits(16..=26, count.into());
        self
    }

    pub fn trigger_flag(&self) -> Trigger {
        self.0.get_bit(31).into()
    }

    pub fn with_trigger_flag(&mut self, trigger: Trigger) -> &mut Self {
        self.0.set_bit(31, trigger.into());
        self
    }
}
