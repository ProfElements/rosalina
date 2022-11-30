use alloc::boxed::Box;
use core::fmt::Write;
use spin::RwLock;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

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

#[derive(EnumIter, Display, Copy, Clone, Debug, PartialEq)]
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

impl Interrupt {
    const COUNT: usize = 15;

    pub fn set_interrupt_handler<F>(interrupt: Interrupt, handler: F)
    where
        F: Fn(usize) -> Result<(), &'static str> + Send + Sync + 'static,
    {
        unsafe {
            writeln!(DOLPHIN_HLE, "Registering {} interrupt handler", interrupt).ok();
        }
        INTERRUPT_TABLE[interrupt.id()].set(handler);
    }

    pub fn invoke_interrupt_handler(interrupt: Interrupt) -> Result<(), &'static str> {
        match INTERRUPT_TABLE[interrupt.id()].f.read().as_ref() {
            Some(f) => f(interrupt.id()),
            None => Ok(()),
        }
    }

    pub fn id(&self) -> usize {
        match self {
            Interrupt::Error => 0,
            Interrupt::ResetSwitch => 1,
            Interrupt::DvdInterface => 2,
            Interrupt::SerialInterface => 3,
            Interrupt::ExternalInterface => 4,
            Interrupt::AudioInterface => 5,
            Interrupt::DSP => 6,
            Interrupt::MemoryInterface => 7,
            Interrupt::VideoInterface => 8,
            Interrupt::PixelEngineToken => 9,
            Interrupt::PixelEngineFinish => 10,
            Interrupt::CommandProcessor => 11,
            Interrupt::Debugger => 12,
            Interrupt::HighSpeedPort => 13,
            Interrupt::InterprocessControl => 14,
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
pub fn interrupt_handler(_addr: usize, _frame: &ExceptionFrame) -> Result<(), &'static str> {
    let cause: InterruptCause = InterruptCause::read();
    let mask: InterruptMask = InterruptMask::read();
    for (idx, interrupt) in Interrupt::iter().enumerate() {
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
            let res = match INTERRUPT_TABLE[idx].f.read().as_ref() {
                Some(f) => f(idx),
                None => Ok(()),
            };

            res?
        }
    }

    Ok(())
}
