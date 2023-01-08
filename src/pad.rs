use bit_field::BitField;

use crate::{
    mmio::{
        pi::Mask,
        si::{SiChannel, SiComm, SiInputBufHi, SiInputBufLo, SiPoll},
        vi::Enabled,
    },
    si::{SerialInterface, TransferError},
};

pub struct Pad {
    kind: u32,
    channel: SiChannel,
}

impl Pad {
    pub fn init(channel: SiChannel) -> Result<Self, TransferError> {
        let kind = SerialInterface::get_type(channel)?;
        match channel {
            SiChannel::Zero => SiPoll::read().with_chan_0_enable(Enabled::Enabled).write(),
            SiChannel::One => SiPoll::read().with_chan_1_enable(Enabled::Enabled).write(),
            SiChannel::Two => SiPoll::read().with_chan_2_enable(Enabled::Enabled).write(),
            SiChannel::Three => SiPoll::read().with_chan_3_enable(Enabled::Enabled).write(),
        }
        SiComm::read()
            .with_read_status_interrupt_mask(Mask::Enabled)
            .write();
        Ok(Self { channel, kind })
    }

    /// # Errors
    ///
    /// This can produce 4 errors.
    /// `NoResponse`: there is no available serial device on that channel
    /// `Overrun`: you had a buffer overrun
    /// `Underrun`: you had a buffer underrun
    /// `Collision`: your values are getting modified while read hopefully this doesnt happen

    pub fn read(&self) -> Status {
        if self.kind & 0x0800_0000 != 0 {
            match self.channel {
                SiChannel::Zero => {
                    return Status::from((SiInputBufHi::read_zero(), SiInputBufLo::read_zero()))
                }
                SiChannel::One => {
                    return Status::from((SiInputBufHi::read_one(), SiInputBufLo::read_one()))
                }
                SiChannel::Two => {
                    return Status::from((SiInputBufHi::read_two(), SiInputBufLo::read_two()))
                }
                SiChannel::Three => {
                    return Status::from((SiInputBufHi::read_three(), SiInputBufLo::read_three()))
                }
            }
        }
        Status::default()
    }
}

#[derive(Default, Debug)]
#[repr(C)]
pub struct Status {
    stick_x: u8,
    stick_y: u8,
    sub_stick_x: u8,
    sub_stick_y: u8,
    analog_l: u8,
    analog_r: u8,
    a: bool,
    b: bool,
    x: bool,
    y: bool,
    start: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    z: bool,
    l: bool,
    r: bool,
}

//All bit gets are ganureteed so its not actually fallible
#[allow(clippy::fallible_impl_from)]
impl From<(SiInputBufHi, SiInputBufLo)> for Status {
    fn from(value: (SiInputBufHi, SiInputBufLo)) -> Self {
        let hi: u32 = u32::from(value.0);
        let lo: u32 = u32::from(value.1);
        Self {
            a: hi.get_bit(24),
            b: hi.get_bit(25),
            x: hi.get_bit(26),
            y: hi.get_bit(27),
            start: hi.get_bit(28),
            z: hi.get_bit(29),
            left: hi.get_bit(16),
            right: hi.get_bit(17),
            down: hi.get_bit(18),
            up: hi.get_bit(19),
            r: hi.get_bit(20),
            l: hi.get_bit(21),
            stick_x: hi.get_bits(8..=15).try_into().unwrap(),
            stick_y: hi.get_bits(0..=7).try_into().unwrap(),
            analog_r: lo.get_bits(0..=7).try_into().unwrap(),
            analog_l: lo.get_bits(8..=15).try_into().unwrap(),
            sub_stick_y: lo.get_bits(16..=23).try_into().unwrap(),
            sub_stick_x: lo.get_bits(24..=31).try_into().unwrap(),
        }
    }
}

impl Status {
    pub const fn a(&self) -> bool {
        self.a
    }
    pub const fn b(&self) -> bool {
        self.b
    }

    pub const fn x(&self) -> bool {
        self.x
    }

    pub const fn y(&self) -> bool {
        self.y
    }

    pub const fn start(&self) -> bool {
        self.start
    }

    pub const fn z(&self) -> bool {
        self.z
    }

    pub const fn l(&self) -> bool {
        self.l
    }

    pub const fn r(&self) -> bool {
        self.r
    }

    pub const fn dpad_up(&self) -> bool {
        self.up
    }

    pub const fn dpad_down(&self) -> bool {
        self.down
    }

    pub const fn dpad_left(&self) -> bool {
        self.left
    }

    pub const fn dpad_right(&self) -> bool {
        self.right
    }

    pub const fn stick(&self) -> Stick {
        Stick {
            x: self.stick_x,
            y: self.stick_y,
        }
    }

    pub const fn sub_stick(&self) -> Stick {
        Stick {
            x: self.sub_stick_x,
            y: self.sub_stick_y,
        }
    }

    pub const fn analog_l(&self) -> u8 {
        self.analog_l
    }

    pub const fn analog(&self) -> u8 {
        self.analog_r
    }
}

pub struct Stick {
    pub x: u8,
    pub y: u8,
}
