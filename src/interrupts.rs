use core::arch::asm;

use alloc::boxed::Box;
use spin::RwLock;
use strum_macros::{Display, EnumIter};

use crate::exception::ExceptionFrame;

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
}

impl Interrupt {
    const COUNT: usize = 14;
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

pub unsafe extern "C" fn interrupt_handler(
    addr: usize,
    frame: &ExceptionFrame,
) -> Result<(), &'static str> {
    let cause = 0u32;
    let mask = 0u32;

    cause = cause & !mask;

    Ok(())
}
