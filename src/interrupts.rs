use alloc::boxed::Box;
use core::fmt::{Display, Write};
use spin::RwLock;

use crate::{
    arch::MachineStateRegister,
    exception::ExceptionFrame,
    mmio::{
        pi::{InterruptCause, InterruptMask},
        vi::Enabled,
    },
    DOLPHIN_HLE,
};

static INTERRUPT_TABLE: [InterruptHandler; Interrupt::COUNT] =
    [const { InterruptHandler::new() }; Interrupt::COUNT];

type DynInterruptHandler = dyn Fn(usize) -> Result<(), &'static str> + Send + Sync + 'static;

pub struct InterruptHandler {
    f: RwLock<Option<Box<DynInterruptHandler>>>,
}

impl InterruptHandler {
    pub const fn new() -> Self {
        Self {
            f: RwLock::new(None),
        }
    }

    pub fn set(&self, f: impl Fn(usize) -> Result<(), &'static str> + Send + Sync + 'static) {
        *self.f.write() = Some(Box::new(f));
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Interrupt {
    Error,
    ResetSwitch,
    DvdInterface,
    SerialInterface,
    ExternalInterface,
    AudioInterface,
    DSP,
    MemoryInterface,
    VideoInterface,
    PixelEngineToken,
    PixelEngineFinish,
    CommandProcessor,
    Debugger,
    HighSpeedPort,
    InterprocessControl,
}

impl Display for Interrupt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let str: &'static str = match self {
            Self::Error => "Error",
            Self::ResetSwitch => "Reset Switch",
            Self::DvdInterface => "DVD Interface",
            Self::SerialInterface => "Serial Interface",
            Self::ExternalInterface => "ExternalInterface",
            Self::AudioInterface => "AudioInterface",
            Self::DSP => "DSP Interface",
            Self::MemoryInterface => "Memory Interface",
            Self::VideoInterface => "Video Interface",
            Self::PixelEngineToken => "Pixel Engine Token",
            Self::PixelEngineFinish => "Pixel Engine Finish",
            Self::CommandProcessor => "Command Processor",
            Self::Debugger => "Debugger",
            Self::HighSpeedPort => "High Speed Port",
            Self::InterprocessControl => "IPC/IOS Control",
        };
        write!(f, "{str}")
    }
}

impl Interrupt {
    const COUNT: usize = 15;

    pub fn set_interrupt_handler<F>(interrupt: Self, handler: F)
    where
        F: Fn(usize) -> Result<(), &'static str> + Send + Sync + 'static,
    {
        unsafe {
            writeln!(DOLPHIN_HLE, "Registering {interrupt} interrupt handler").ok();
        }
        INTERRUPT_TABLE[interrupt.id()].set(handler);
    }

    /// # Errors
    ///
    /// This errors based on user provided function.
    pub fn invoke_interrupt_handler(interrupt: Self) -> Result<(), &'static str> {
        INTERRUPT_TABLE[interrupt.id()]
            .f
            .read()
            .as_ref()
            .map_or(Ok(()), |f| f(interrupt.id()))
    }

    pub const fn id(&self) -> usize {
        match self {
            Self::Error => 0,
            Self::ResetSwitch => 1,
            Self::DvdInterface => 2,
            Self::SerialInterface => 3,
            Self::ExternalInterface => 4,
            Self::AudioInterface => 5,
            Self::DSP => 6,
            Self::MemoryInterface => 7,
            Self::VideoInterface => 8,
            Self::PixelEngineToken => 9,
            Self::PixelEngineFinish => 10,
            Self::CommandProcessor => 11,
            Self::Debugger => 12,
            Self::HighSpeedPort => 13,
            Self::InterprocessControl => 14,
        }
    }

    pub const fn from_id(id: usize) -> Option<Self> {
        match id {
            0 => Some(Self::Error),
            1 => Some(Self::ResetSwitch),
            2 => Some(Self::DvdInterface),
            3 => Some(Self::SerialInterface),
            4 => Some(Self::ExternalInterface),
            5 => Some(Self::AudioInterface),
            6 => Some(Self::DSP),
            7 => Some(Self::MemoryInterface),
            8 => Some(Self::VideoInterface),
            9 => Some(Self::PixelEngineToken),
            10 => Some(Self::PixelEngineFinish),
            11 => Some(Self::CommandProcessor),
            12 => Some(Self::Debugger),
            13 => Some(Self::HighSpeedPort),
            14 => Some(Self::InterprocessControl),
            _ => None,
        }
    }
}

pub fn disable() {
    MachineStateRegister::read()
        .with_external_interrupt_enabled(Enabled::Disabled)
        .write();
}

pub fn enable() {
    MachineStateRegister::read()
        .with_external_interrupt_enabled(Enabled::Enabled)
        .write();
}

/// # Errors
///
/// This errors based on user provided function.

pub fn interrupt_handler(_addr: usize, _frame: &ExceptionFrame) -> Result<(), &'static str> {
    let cause: InterruptCause = InterruptCause::read();
    let mask: InterruptMask = InterruptMask::read();
    for (n, func) in INTERRUPT_TABLE.iter().enumerate().take(Interrupt::COUNT) {
        let interrupt = Interrupt::from_id(n).unwrap();
        let is_enabled: bool = match interrupt {
            Interrupt::Error => cause.gp_runtime_error().into() && mask.gp_runtime_error().into(),
            Interrupt::ResetSwitch => cause.reset_switch().into() && mask.reset_switch().into(),
            Interrupt::DvdInterface => cause.dvd_interface().into() && mask.dvd_interface().into(),
            Interrupt::SerialInterface => {
                cause.serial_interface().into() && mask.serial_interface().into()
            }
            Interrupt::ExternalInterface => {
                cause.external_interface().into() && mask.external_interface().into()
            }
            Interrupt::AudioInterface => {
                cause.audio_interface().into() && mask.audio_interface().into()
            }
            Interrupt::DSP => cause.dsp_interface().into() && mask.dsp_interface().into(),
            Interrupt::MemoryInterface => {
                cause.memory_interface().into() && mask.memory_interface().into()
            }
            Interrupt::VideoInterface => {
                cause.video_interface().into() && mask.video_interface().into()
            }
            Interrupt::PixelEngineToken => {
                cause.pixel_engine_token().into() && mask.pixel_engine_token().into()
            }
            Interrupt::PixelEngineFinish => {
                cause.pixel_engine_finish().into() && mask.pixel_engine_finish().into()
            }
            Interrupt::CommandProcessor => {
                cause.command_fifo().into() && mask.command_fifo().into()
            }
            Interrupt::Debugger => cause.debug().into() && mask.debug().into(),
            Interrupt::HighSpeedPort => {
                cause.high_speed_port().into() && mask.high_speed_port().into()
            }
            Interrupt::InterprocessControl => {
                cause.interprocess_control().into() && mask.interprocess_control().into()
            }
        };

        if is_enabled {
            let res = func.f.read().as_ref().map_or(Ok(()), |f| f(n));

            res?;
        }
    }

    Ok(())
}
