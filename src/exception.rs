use core::arch::asm;

use alloc::boxed::Box;
use spin::RwLock;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use crate::rt0::{
    data_cache_flush_range_no_sync, instruction_cache_invalidate_range, EXCEPTION_HANDLER_END,
    EXCEPTION_HANDLER_START, SYSTEMCALL_HANDLER_END, SYSTEMCALL_HANDLER_START,
};

use crate::print;

static EXCEPTION_TABLE: [ExceptionHandler; Exception::COUNT] =
    [const { ExceptionHandler::new() }; Exception::COUNT];

type DynExceptionHandler =
    dyn Fn(usize, &ExceptionFrame) -> Result<(), &'static str> + Send + Sync + 'static;

pub struct ExceptionHandler {
    f: RwLock<Option<Box<DynExceptionHandler>>>,
}

impl ExceptionHandler {
    pub const fn new() -> Self {
        Self {
            f: RwLock::new(None),
        }
    }

    pub fn set(
        &self,
        f: impl Fn(usize, &ExceptionFrame) -> Result<(), &'static str> + Send + Sync + 'static,
    ) {
        *self.f.write() = Some(Box::new(f));
    }
}

#[derive(Default)]
pub struct ExceptionFrame {
    srr0: u32,
    srr1: u32,
    gprs: [u32; 32],
    gqrs: [u32; 8],
    cr: u32,
    lr: u32,
    ctr: u32,
    xer: u32,
    msr: u32,
    dar: u32,

    state: u16,

    fprs: [f64; 32],
    psfprs: [f64; 32],
    fpscr: u64,
}

#[derive(EnumIter, Display, Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum Exception {
    #[strum(serialize = "System Reset")]
    SystemReset,
    #[strum(serialize = "Machine Check")]
    MachineCheck,
    Dsi,
    Isi,
    Interrupt,
    Alignment,
    Program,
    #[strum(serialize = "Floating Point")]
    FloatingPoint,
    Decrementer,
    #[strum(serialize = "System Call")]
    SystemCall,
    Trace,
    Performance,
    Iabr,
    Reserved,
    Thermal,
}

impl Exception {
    const COUNT: usize = 15;
    pub fn id(&self) -> usize {
        match self {
            Exception::SystemReset => 0,
            Exception::MachineCheck => 1,
            Exception::Dsi => 2,
            Exception::Isi => 3,
            Exception::Interrupt => 4,
            Exception::Alignment => 5,
            Exception::Program => 6,
            Exception::FloatingPoint => 7,
            Exception::Decrementer => 8,
            Exception::SystemCall => 9,
            Exception::Trace => 10,
            Exception::Performance => 11,
            Exception::Iabr => 12,
            Exception::Reserved => 13,
            Exception::Thermal => 14,
        }
    }

    pub fn addr(&self) -> usize {
        match self {
            Exception::SystemReset => 0x80000100,
            Exception::MachineCheck => 0x80000200,
            Exception::Dsi => 0x80000300,
            Exception::Isi => 0x80000400,
            Exception::Interrupt => 0x80000500,
            Exception::Alignment => 0x80000600,
            Exception::Program => 0x80000700,
            Exception::FloatingPoint => 0x80000800,
            Exception::Decrementer => 0x80000900,
            Exception::SystemCall => 0x80000C00,
            Exception::Trace => 0x80000D00,
            Exception::Performance => 0x80000F00,
            Exception::Iabr => 0x80001300,
            Exception::Reserved => 0x80001400,
            Exception::Thermal => 0x80001700,
        }
    }

    pub fn from_id(id: usize) -> Option<Self> {
        match id {
            0 => Some(Exception::SystemReset),
            1 => Some(Exception::MachineCheck),
            2 => Some(Exception::Dsi),
            3 => Some(Exception::Isi),
            4 => Some(Exception::Interrupt),
            5 => Some(Exception::Alignment),
            6 => Some(Exception::Program),
            7 => Some(Exception::FloatingPoint),
            8 => Some(Exception::Decrementer),
            9 => Some(Exception::SystemCall),
            10 => Some(Exception::Trace),
            11 => Some(Exception::Performance),
            12 => Some(Exception::Iabr),
            13 => Some(Exception::Reserved),
            14 => Some(Exception::Thermal),
            _ => None,
        }
    }

    pub fn from_addr(addr: usize) -> Option<Self> {
        match addr {
            0x80000100 => Some(Exception::SystemReset),
            0x80000200 => Some(Exception::MachineCheck),
            0x80000300 => Some(Exception::Dsi),
            0x80000400 => Some(Exception::Isi),
            0x80000500 => Some(Exception::Interrupt),
            0x80000600 => Some(Exception::Alignment),
            0x80000700 => Some(Exception::Program),
            0x80000800 => Some(Exception::FloatingPoint),
            0x80000900 => Some(Exception::Decrementer),
            0x80000C00 => Some(Exception::SystemCall),
            0x80000D00 => Some(Exception::Trace),
            0x80000F00 => Some(Exception::Performance),
            0x80001300 => Some(Exception::Iabr),
            0x80001400 => Some(Exception::Reserved),
            0x80001700 => Some(Exception::Thermal),
            _ => None,
        }
    }
}

pub struct ExceptionSystem;

impl ExceptionSystem {
    pub fn init() {
        for exception in Exception::iter() {
            if exception == Exception::SystemCall {
                unsafe {
                    Self::load_exception_handler(
                        exception,
                        SYSTEMCALL_HANDLER_START.as_ptr(),
                        SYSTEMCALL_HANDLER_END.as_usize() - SYSTEMCALL_HANDLER_START.as_usize(),
                    );
                }
                continue;
            }

            unsafe {
                Self::load_exception_handler(
                    exception,
                    EXCEPTION_HANDLER_START.as_ptr(),
                    EXCEPTION_HANDLER_END.as_usize() - EXCEPTION_HANDLER_START.as_usize(),
                );
            }
            Self::set_exception_handler(exception, default_exception_handler);
        }
    }

    pub unsafe fn load_exception_handler(
        exception: Exception,
        asm_start: *const u8,
        asm_len: usize,
    ) {
        let addr = exception.addr();
        let addr_ptr = addr as *mut u8;

        print!(
            "Loading exception handler for  {} Exception at address: {}\n",
            exception, addr
        );

        core::ptr::copy_nonoverlapping(asm_start, addr_ptr, asm_len);
        data_cache_flush_range_no_sync(addr_ptr, asm_len.try_into().unwrap());
        instruction_cache_invalidate_range(addr_ptr, asm_len.try_into().unwrap());

        core::arch::asm!("sync");
    }

    pub fn set_exception_handler<F>(exception: Exception, handler: F)
    where
        F: Fn(usize, &ExceptionFrame) -> Result<(), &'static str> + Send + Sync + 'static,
    {
        print!(
            "Registering exception handler for {} Exception at address: {}\n",
            exception,
            exception.addr()
        );

        EXCEPTION_TABLE[exception.id()].set(handler);
    }

    pub fn invoke_exception_handler(exception: Exception, frame: &ExceptionFrame) -> Result<(), &'static str> {
        match EXCEPTION_TABLE[exception.id()].f.read().as_ref() {
            Some(f) => f(exception.id(), frame),
            None => Ok(()),
        }
    }
}

pub fn default_exception_handler(
    exception_id: usize,
    frame: &ExceptionFrame,
) -> Result<(), &'static str> {
    print!(
        "Exception {} has occured!",
        Exception::from_id(exception_id).unwrap()
    );

    // PRINT REGISTERS
    print!(
        "GPR00 {:X?}, GPR08 {:X?}, GPR16 {:X?}, GPR24: {:X?}\n",
        frame.gprs[0], frame.gprs[8], frame.gprs[16], frame.gprs[24]
    );
    print!(
        "GPR01 {:X?}, GPR09 {:X?}, GPR17 {:X?}, GPR25: {:X?}\n",
        frame.gprs[1], frame.gprs[9], frame.gprs[17], frame.gprs[25]
    );
    print!(
        "GPR02 {:X?}, GPR10 {:X?}, GPR18 {:X?}, GPR26: {:X?}\n",
        frame.gprs[2], frame.gprs[10], frame.gprs[18], frame.gprs[26]
    );
    print!(
        "GPR03 {:X?}, GPR11 {:X?}, GPR19 {:X?}, GPR27: {:X?}\n",
        frame.gprs[3], frame.gprs[11], frame.gprs[19], frame.gprs[27]
    );
    print!(
        "GPR04 {:X?}, GPR12 {:X?}, GPR20 {:X?}, GPR28: {:X?}\n",
        frame.gprs[4], frame.gprs[12], frame.gprs[20], frame.gprs[28]
    );
    print!(
        "GPR05 {:X?}, GPR13 {:X?}, GPR21 {:X?}, GPR29: {:X?}\n",
        frame.gprs[5], frame.gprs[13], frame.gprs[21], frame.gprs[29]
    );
    print!(
        "GPR06 {:X?}, GPR14 {:X?}, GPR22 {:X?}, GPR30: {:X?}\n",
        frame.gprs[6], frame.gprs[14], frame.gprs[22], frame.gprs[30]
    );
    print!(
        "GPR07 {:X?}, GPR15 {:X?}, GPR23 {:X?}, GPR31: {:X?}\n",
        frame.gprs[7], frame.gprs[15], frame.gprs[23], frame.gprs[31]
    );

    print!(
        "LR: {:X?}, SRR0: {:X?}, SRR1: {:X?}, MSR: {:X?}\n",
        frame.lr, frame.srr0, frame.srr1, frame.msr
    );
    print!("DAR: {:X?}, DSISR: {:X?}\n", mfspr(19), mfspr(18));

    Err("An Exception has Occured")
}

fn mfspr(spr: i32) -> i32 {
    let mut _outspr = 0;

    unsafe {
        asm!("mfspr {0},{1}", out(reg) _outspr, in(reg)spr);
    }
    _outspr
}

#[inline(never)]
#[no_mangle]
//TODO: Get a proper exception frame instead of a junk one from a random pointer :shrug:
pub unsafe extern "C" fn exception_handler(mut addr: usize, frame: &mut ExceptionFrame) {
    if addr < 0x80000000 { addr += 0x80000000 } 
    if let Some(exception) = Exception::from_addr(addr) {
        let _ = ExceptionSystem::invoke_exception_handler(exception, &frame).unwrap();   
    }
    core::hint::unreachable_unchecked();
}
