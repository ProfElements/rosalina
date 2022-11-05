use alloc::boxed::Box;
use core::arch::asm;
use core::fmt::Write;
use spin::RwLock;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use crate::{
    exception::ExceptionFrame,
    mmio::pi::{InterruptCause, InterruptMask},
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
        INTERRUPT_TABLE[(interrupt as u32) as usize].set(handler);
    }

    pub fn invoke_interrupt_handler(interrupt: Interrupt) -> Result<(), &'static str> {
        match INTERRUPT_TABLE[interrupt as u32 as usize].f.read().as_ref() {
            Some(f) => f(interrupt as u32 as usize),
            None => Ok(()),
        }
    }
}

pub fn disable() -> usize {
    let mut cookie = 0usize;
    unsafe {
        asm!("mfmsr 3",
          "rlwinm. 3,{0},0,17,15",
          "mtmsr 3",
          "extrwi. {0},{0},1,16",
          inout(reg) cookie,
        );
    }
    cookie
}

pub fn enable() {
    unsafe {
        asm!("mfmsr 3", "ori 3,3,0x8000", "mtmsr 3");
    }
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
