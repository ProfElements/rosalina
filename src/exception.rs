use core::arch::asm;

use alloc::boxed::Box;
use spin::RwLock;

use crate::rt0::{EXCEPTION_HANDLER_START, EXCEPTION_HANDLER_END, SYSTEMCALL_HANDLER_START, SYSTEMCALL_HANDLER_END, data_cache_flush_range_no_sync, instruction_cache_invalidate_range};

const NUM_EXCEPTIONS: usize = 15;

static EXCEPTION_TABLE: [ExceptionHandler; NUM_EXCEPTIONS] =
    [const { ExceptionHandler::new() }; NUM_EXCEPTIONS];

pub enum Exception {
    SystemReset,
    MachineCheck,
    Dsi,
    Isi,
    Interrupt,
    Alignment,
    Program,
    FloatingPoint,
    Decremeter,
    SystemCall,
    Trace,
    Performance,
    Iabr,
    Reserved,
    Thermal,
}

impl Exception {
    fn from_id(id: usize) -> Self {
        match id {
            0 => Self::SystemReset,
            1 => Self::MachineCheck,
            2 => Self::Dsi,
            3 => Self::Isi,
            4 => Self::Interrupt,
            5 => Self::Alignment,
            6 => Self::Program,
            7 => Self::FloatingPoint,
            8 => Self::Decremeter,
            9 => Self::SystemCall,
            10 => Self::Trace,
            11 => Self::Performance,
            12 => Self::Iabr,
            13 => Self::Reserved,
            14 => Self::Thermal,
            _ => Self::SystemReset,
        }
    }
    fn into_id(&self) -> usize {
        match self {
            Exception::SystemReset => 0,
            Exception::MachineCheck => 1,
            Exception::Dsi => 2,
            Exception::Isi => 3,
            Exception::Interrupt => 4,
            Exception::Alignment => 5,
            Exception::Program => 6,
            Exception::FloatingPoint => 7,
            Exception::Decremeter => 8,
            Exception::SystemCall => 9,
            Exception::Trace => 10,
            Exception::Performance => 11,
            Exception::Iabr => 12,
            Exception::Reserved => 13,
            Exception::Thermal => 14,
        }
    }
   pub fn name(&self) -> &'static str {
        match self {
            Self::SystemReset => "System Reset",
            Self::MachineCheck => "Machine Check",
            Self::Dsi => "DSI",
            Self::Isi => "ISI",
            Self::Interrupt => "Interrupt",
            Self::Alignment => "Alignment",
            Self::Program => "Program",
            Self::FloatingPoint => "Floating Point",
            Self::Decremeter => "Decremeter",
            Self::SystemCall => "System Call",
            Self::Trace => "Trace",
            Self::Performance => "Performance",
            Self::Iabr => "IABR",
            Self::Reserved => "Reserved",
            Self::Thermal => "Thermal",
        }
    }

    pub fn loc(&self) -> usize {
        match self {
            Self::SystemReset =>   0x80000100,
            Self::MachineCheck =>  0x80000200,
            Self::Dsi =>           0x80000300,
            Self::Isi =>           0x80000400,
            Self::Interrupt =>     0x80000500,
            Self::Alignment =>     0x80000600,
            Self::Program =>       0x80000700,
            Self::FloatingPoint => 0x80000800,
            Self::Decremeter =>    0x80000900,
            Self::SystemCall =>    0x80000C00,
            Self::Trace =>         0x80000D00,
            Self::Performance =>   0x80000F00,
            Self::Iabr =>          0x80001300,
            Self::Reserved =>      0x80001400,
            Self::Thermal =>       0x80001700,
        }
    }
}

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

pub fn register_exception_handler<F>(exception_id: usize, f: F)
where
    F: Fn(usize, &ExceptionFrame) -> Result<(), &'static str> + Send + Sync + 'static,
{
    print!(
        "Registering exception handler for {} Exception",
        Exception::from_id(exception_id).name()
    );
    EXCEPTION_TABLE[exception_id].set(f);
}

pub fn invoke_exception_handler(
    interrupt_id: usize,
    frame: &ExceptionFrame,
) -> Result<(), &'static str> {
    match EXCEPTION_TABLE[interrupt_id].f.read().as_ref() {
        Some(f) => f(interrupt_id, frame),
        None => Ok(()),
    }
}

pub fn _default_exception_handler(
    exception_id: usize,
    frame: &ExceptionFrame,
) -> Result<(), &'static str> {
    print!(
        "Exception {} has occured!",
        Exception::from_id(exception_id).name()
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

    //PRINT STACK SOMEHOW

    //DSI OR FP STUFF
    //

    Err("An Exception has Occured")
}

fn mfspr(spr: i32) -> i32 {
    let mut outspr = 0;

    unsafe {
        asm!("mfspr {0},{1}", out(reg) outspr, in(reg)spr);
    }
    outspr
}
/*
extern "C" {
    pub static EXCEPTION_HANDLER_START: LinkerSymbol;
    pub static EXCEPTION_HANDLER_END: LinkerSymbol;
}
*/

pub unsafe fn exception_init() {
    for n in 0..NUM_EXCEPTIONS {
        exception_load(
            Exception::from_id(n),
            EXCEPTION_HANDLER_START.as_ptr(),
            EXCEPTION_HANDLER_END.as_usize() - EXCEPTION_HANDLER_START.as_usize(),
            core::ptr::null(),
        );
    }
}

unsafe fn exception_load(exception: Exception, data: *const u8, len: usize, _patch: *const u8) {
    let addr = exception.loc();
    print!("Loading Exception: {} at location: {:02X?}\n", exception.name(), addr);
    let len = len + 2;
    core::ptr::copy_nonoverlapping(data, addr as *const u8 as *mut u8, len);
    data_cache_flush_range_no_sync(addr as *const u8, len as u32);
    instruction_cache_invalidate_range(addr as *const u8, len as u32);
    asm!("sync");
}

pub unsafe fn systemcall_init() {
   exception_load(Exception::SystemCall, SYSTEMCALL_HANDLER_START.as_ptr(), SYSTEMCALL_HANDLER_END.as_usize() - SYSTEMCALL_HANDLER_START.as_usize(), core::ptr::null()); 
}

#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn exception_handler() {
    loop {}
}
