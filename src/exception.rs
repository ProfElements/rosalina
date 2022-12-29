use core::fmt::Display;
use core::{arch::asm, fmt::Write};

use alloc::boxed::Box;
use spin::RwLock;

use crate::cache::{dc_flush_range_no_sync, ic_invalidate_range};
use crate::interrupts::interrupt_handler;

use crate::os::LinkerSymbol;
use crate::DOLPHIN_HLE;

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

#[derive(Default, Debug)]
#[repr(C)]
pub struct ExceptionFrame {
    gprs: [u32; 32],
    srr0: u32,
    srr1: u32,
    cr: u32,
    lr: u32,
    ctr: u32,
    xer: u32,
    msr: u32,
    dar: u32,

    state: u16,

    gqrs: [u32; 8],
    fprs: [f64; 32],
    psfprs: [f64; 32],
    fpscr: u64,
}

impl ExceptionFrame {
    pub const fn new() -> Self {
        Self {
            srr0: 0,
            srr1: 0,
            gprs: [0; 32],
            gqrs: [0; 8],
            cr: 0,
            lr: 0,
            ctr: 0,
            xer: 0,
            msr: 0,
            dar: 0,
            state: 0,
            fprs: [0.0; 32],
            psfprs: [0.0; 32],
            fpscr: 0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Exception {
    SystemReset,
    MachineCheck,
    Dsi,
    Isi,
    Interrupt,
    Alignment,
    Program,
    FloatingPoint,
    Decrementer,
    SystemCall,
    Trace,
    Performance,
    Iabr,
    Reserved,
    Thermal,
}

impl Display for Exception {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let str: &'static str = match self {
            Self::SystemReset => "System Reset",
            Self::MachineCheck => "Machine Check",
            Self::Dsi => "DSI",
            Self::Isi => "ISI",
            Self::Interrupt => "Interrupt",
            Self::Alignment => "Alignment",
            Self::Program => "Program",
            Self::FloatingPoint => "Floating Point",
            Self::Decrementer => "Decrementer",
            Self::SystemCall => "System Call",
            Self::Trace => "Trace",
            Self::Performance => "Performance",
            Self::Iabr => "IABR",
            Self::Reserved => "Reserved",
            Self::Thermal => "Thermal",
        };
        write!(f, "{str}")
    }
}

impl Exception {
    const COUNT: usize = 15;
    pub const fn id(&self) -> usize {
        match self {
            Self::SystemReset => 0,
            Self::MachineCheck => 1,
            Self::Dsi => 2,
            Self::Isi => 3,
            Self::Interrupt => 4,
            Self::Alignment => 5,
            Self::Program => 6,
            Self::FloatingPoint => 7,
            Self::Decrementer => 8,
            Self::SystemCall => 9,
            Self::Trace => 10,
            Self::Performance => 11,
            Self::Iabr => 12,
            Self::Reserved => 13,
            Self::Thermal => 14,
        }
    }

    pub const fn addr(&self) -> usize {
        match self {
            Self::SystemReset => 0x80000100,
            Self::MachineCheck => 0x80000200,
            Self::Dsi => 0x80000300,
            Self::Isi => 0x80000400,
            Self::Interrupt => 0x80000500,
            Self::Alignment => 0x80000600,
            Self::Program => 0x80000700,
            Self::FloatingPoint => 0x80000800,
            Self::Decrementer => 0x80000900,
            Self::SystemCall => 0x80000C00,
            Self::Trace => 0x80000D00,
            Self::Performance => 0x80000F00,
            Self::Iabr => 0x80001300,
            Self::Reserved => 0x80001400,
            Self::Thermal => 0x80001700,
        }
    }

    pub const fn from_id(id: usize) -> Option<Self> {
        match id {
            0 => Some(Self::SystemReset),
            1 => Some(Self::MachineCheck),
            2 => Some(Self::Dsi),
            3 => Some(Self::Isi),
            4 => Some(Self::Interrupt),
            5 => Some(Self::Alignment),
            6 => Some(Self::Program),
            7 => Some(Self::FloatingPoint),
            8 => Some(Self::Decrementer),
            9 => Some(Self::SystemCall),
            10 => Some(Self::Trace),
            11 => Some(Self::Performance),
            12 => Some(Self::Iabr),
            13 => Some(Self::Reserved),
            14 => Some(Self::Thermal),
            _ => None,
        }
    }

    pub const fn from_addr(addr: usize) -> Option<Self> {
        match addr {
            0x80000100 => Some(Self::SystemReset),
            0x80000200 => Some(Self::MachineCheck),
            0x80000300 => Some(Self::Dsi),
            0x80000400 => Some(Self::Isi),
            0x80000500 => Some(Self::Interrupt),
            0x80000600 => Some(Self::Alignment),
            0x80000700 => Some(Self::Program),
            0x80000800 => Some(Self::FloatingPoint),
            0x80000900 => Some(Self::Decrementer),
            0x80000C00 => Some(Self::SystemCall),
            0x80000D00 => Some(Self::Trace),
            0x80000F00 => Some(Self::Performance),
            0x80001300 => Some(Self::Iabr),
            0x80001400 => Some(Self::Reserved),
            0x80001700 => Some(Self::Thermal),
            _ => None,
        }
    }

    pub fn init() {
        for n in 0..Self::COUNT {
            let exception = Self::from_id(n).unwrap();
            if exception == Self::SystemCall {
                unsafe {
                    Self::load_exception_handler(
                        exception,
                        SYSTEMCALL_HANDLER_START.as_ptr(),
                        SYSTEMCALL_HANDLER_END.as_usize() - SYSTEMCALL_HANDLER_START.as_usize(),
                    );
                }
                continue;
            }
            if exception == Self::Decrementer || exception == Self::Interrupt {
                unsafe {
                    Self::load_exception_handler(
                        exception,
                        RECOVERABLE_HANDLER_START.as_ptr(),
                        RECOVERABLE_HANDLER_END.as_usize() - RECOVERABLE_HANDLER_START.as_usize(),
                    );
                }
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
        Self::set_exception_handler(Self::Interrupt, interrupt_handler);
    }

    /// # Safety
    ///
    /// The caller must provide valid pointers to this function.
    /// The called must provide correct length to this function;
    /// This must be called when exceptions are not on.
    pub unsafe fn load_exception_handler(exception: Self, asm_start: *const u8, asm_len: usize) {
        let addr = exception.addr();
        //This size and pointer is always avialable to the powerpc system
        let addr_ptr: *mut [u8; 0x100] = core::ptr::from_exposed_addr_mut(addr);

        writeln!(
            DOLPHIN_HLE,
            "Loading exception handler for {exception} Exception at address: {addr:X?}",
        )
        .ok();

        core::ptr::copy_nonoverlapping(asm_start, addr_ptr.cast::<u8>(), asm_len);
        dc_flush_range_no_sync(addr_ptr.cast::<u8>(), asm_len);
        ic_invalidate_range(addr_ptr.cast::<u8>(), asm_len);
        core::arch::asm!("sync");
    }

    pub fn set_exception_handler<F>(exception: Self, handler: F)
    where
        F: Fn(usize, &ExceptionFrame) -> Result<(), &'static str> + Send + Sync + 'static,
    {
        unsafe {
            writeln!(
                DOLPHIN_HLE,
                "Registering {} exception handler at address: {:X?}",
                exception,
                exception.addr()
            )
            .ok();
        }
        EXCEPTION_TABLE[exception.id()].set(handler);
    }

    /// # Errors
    ///
    /// Returns an error when you encounter an unrecoverable exceptions
    /// Returns an error when an error is provided via a user function
    pub fn invoke_exception_handler(
        exception: Self,
        frame: &ExceptionFrame,
    ) -> Result<(), &'static str> {
        EXCEPTION_TABLE[exception.id()]
            .f
            .read()
            .as_ref()
            .map_or(Ok(()), |f| f(exception.id(), frame))
    }
}
/// # Errors
///
/// Returns an error when you encounter an unrecoverable exception
pub fn default_exception_handler(
    exception_id: usize,
    frame: &ExceptionFrame,
) -> Result<(), &'static str> {
    unsafe {
        writeln!(
            DOLPHIN_HLE,
            "Exception {} has occured!",
            Exception::from_id(exception_id).unwrap()
        )
        .ok();

        // PRINT REGISTERS
        writeln!(
            DOLPHIN_HLE,
            "GPR00 {:X?}, GPR08 {:X?}, GPR16 {:X?}, GPR24: {:X?}",
            frame.gprs[0], frame.gprs[8], frame.gprs[16], frame.gprs[24]
        )
        .ok();
        writeln!(
            DOLPHIN_HLE,
            "GPR01 {:X?}, GPR09 {:X?}, GPR17 {:X?}, GPR25: {:X?}",
            frame.gprs[1], frame.gprs[9], frame.gprs[17], frame.gprs[25]
        )
        .ok();
        writeln!(
            DOLPHIN_HLE,
            "GPR02 {:X?}, GPR10 {:X?}, GPR18 {:X?}, GPR26: {:X?}",
            frame.gprs[2], frame.gprs[10], frame.gprs[18], frame.gprs[26]
        )
        .ok();
        writeln!(
            DOLPHIN_HLE,
            "GPR03 {:X?}, GPR11 {:X?}, GPR19 {:X?}, GPR27: {:X?}",
            frame.gprs[3], frame.gprs[11], frame.gprs[19], frame.gprs[27]
        )
        .ok();
        writeln!(
            DOLPHIN_HLE,
            "GPR04 {:X?}, GPR12 {:X?}, GPR20 {:X?}, GPR28: {:X?}",
            frame.gprs[4], frame.gprs[12], frame.gprs[20], frame.gprs[28]
        )
        .ok();
        writeln!(
            DOLPHIN_HLE,
            "GPR05 {:X?}, GPR13 {:X?}, GPR21 {:X?}, GPR29: {:X?}",
            frame.gprs[5], frame.gprs[13], frame.gprs[21], frame.gprs[29]
        )
        .ok();
        writeln!(
            DOLPHIN_HLE,
            "GPR06 {:X?}, GPR14 {:X?}, GPR22 {:X?}, GPR30: {:X?}",
            frame.gprs[6], frame.gprs[14], frame.gprs[22], frame.gprs[30]
        )
        .ok();
        writeln!(
            DOLPHIN_HLE,
            "GPR07 {:X?}, GPR15 {:X?}, GPR23 {:X?}, GPR31: {:X?}",
            frame.gprs[7], frame.gprs[15], frame.gprs[23], frame.gprs[31]
        )
        .ok();

        writeln!(
            DOLPHIN_HLE,
            "LR: {:X?}, SRR0: {:X?}, SRR1: {:X?}, MSR: {:X?}",
            frame.lr, frame.srr0, frame.srr1, frame.msr
        )
        .ok();
        writeln!(
            DOLPHIN_HLE,
            "DAR: {:X?}, DSISR: {:X?}",
            frame.dar,
            mfspr(18)
        )
        .ok();
    }
    Err("An Unrecoverable exception occured!")
}

fn mfspr(spr: i32) -> i32 {
    let outspr;

    unsafe {
        asm!("mfspr {0},{1}", out(reg)  outspr, in(reg)spr);
    }

    outspr
}

/*
#[inline(never)]
#[no_mangle]
//TODO: Get a proper exception frame instead of a junk one from a random pointer :shrug:
pub unsafe extern "C" fn exception_handler(mut addr: usize, frame: &mut ExceptionFrame) {
    if addr < 0x80000000 {
        addr += 0x80000000
    }
    if let Some(exception) = Exception::from_addr(addr) {
        let _ = Exception::invoke_exception_handler(exception, &frame).unwrap();
    }
    core::hint::unreachable_unchecked();
}
*/

extern "C" {
    static SYSTEMCALL_HANDLER_START: LinkerSymbol;
    static SYSTEMCALL_HANDLER_END: LinkerSymbol;
    static EXCEPTION_HANDLER_START: LinkerSymbol;
    static EXCEPTION_HANDLER_END: LinkerSymbol;
    static RECOVERABLE_HANDLER_START: LinkerSymbol;
    static RECOVERABLE_HANDLER_END: LinkerSymbol;
}

#[naked]
#[allow(named_asm_labels)]
pub extern "C" fn systemcall_handler() {
    unsafe {
        core::arch::asm!(
            ".global SYSTEMCALL_HANDLER_START",
            "SYSTEMCALL_HANDLER_START:",
            "mtspr {SPRG2},9",
            "mtspr {SPRG3},10",
            "mfspr 9,{HID0}",
            "ori 10,9,0x0008",
            "mtspr {HID0},10",
            "isync",
            "sync",
            "mtspr {HID0},9",
            "mfspr 9,{SPRG2}",
            "mfspr 10,{SPRG3}",
            "rfi",
            ".global SYSTEMCALL_HANDLER_END",
            "SYSTEMCALL_HANDLER_END:",
            "nop",
            SPRG2 = const 274,
            SPRG3 = const 275,
            HID0 = const 1008,
            options(noreturn),
        )
    }
}

static mut CONTEXT: ExceptionFrame = ExceptionFrame::new();

#[naked]
#[allow(named_asm_labels)]
pub extern "C" fn exception_handler() {
    unsafe {
        core::arch::asm!(
            ".global EXCEPTION_HANDLER_START",
            "EXCEPTION_HANDLER_START:",
            "mtspr {SPRG3},4",
            "lis 4,{CONTEXT}@h",
            "ori 4,4,{CONTEXT}@l",
            "clrlwi 4,4,2",
            //STORE CONTEXT
            "stw 0,0(4)",
            "stw 1,4(4)",
            "stw 2,8(4)",
            "stw 3,12(4)",
            "mfspr 3,{SPRG3}",
            "stw 3,16(4)",
            "stw 5,20(4)",
            "mfsrr0 3",
            "stw 3,128(4)",
            "mfsrr1 3",
            "stw 3,132(4)",
            "mfcr 3",
            "stw 3,136(4)",
            "mflr 3",
            "stw 3,140(4)",
            "mfctr 3",
            "stw 3,144(4)",
            "mfxer 3",
            "stw 3,148(4)",
            "mfmsr 3",
            "stw 3,152(4)",
            "mfdar 3",
            "stw 3,156(4)",
            //END STORE CONTEXT
            "lis 3,default@h",
            "ori 3,3,default@l",
            "mtsrr0 3",
            "mfmsr 3",
            "ori 3,3,{MSR_DR}|{MSR_IR}|{MSR_FP}",
            "mtsrr1 3",
            "bl 1f",
            "1:",
            "mflr 3",
            "subi 3,3,0x88",
            "rfi",
            ".global EXCEPTION_HANDLER_END",
            "EXCEPTION_HANDLER_END:",
            "nop",
            "default:",
            "lis 4,{CONTEXT}@h",
            "ori 4,4,{CONTEXT}@l",
            "stw 6,24(4)",
            "stw 7,28(4)",
            "stw 8,32(4)",
            "stw 9,36(4)",
            "stw 10,40(4)",
            "stw 11,44(4)",
            "stw 12,48(4)",
            "stw 13,52(4)",
            "stw 14,56(4)",
            "stw 15,60(4)",
            "stw 16,64(4)",
            "stw 17,68(4)",
            "stw 18,72(4)",
            "stw 19,76(4)",
            "stw 20,80(4)",
            "stw 21,84(4)",
            "stw 22,88(4)",
            "stw 23,92(4)",
            "stw 24,96(4)",
            "stw 25,100(4)",
            "stw 26,104(4)",
            "stw 27,108(4)",
            "stw 28,112(4)",
            "stw 29,116(4)",
            "stw 30,120(4)",
            "stw 31,124(4)",
            "bl {default_exception}",
            "lis 4,{CONTEXT}@h",
            "ori 4,4,{CONTEXT}@l",
            "lwz 3,136(4)",
            "mtcr 3",
            "lwz 3,140(4)",
            "mtlr 3",
            "lwz 3,144(4)",
            "mtctr 3",
            "lwz 3,148(4)",
            "mtxer 3",
            "lwz 6,24(4)",
            "lwz 7,28(4)",
            "lwz 8,32(4)",
            "lwz 9,36(4)",
            "lwz 10,40(4)",
            "lwz 11,44(4)",
            "lwz 12,48(4)",
            "lwz 13,52(4)",
            "lwz 14,56(4)",
            "lwz 15,60(4)",
            "lwz 16,64(4)",
            "lwz 0,0(4)",
            "lwz 2,8(4)",
            "lwz 3,128(4)",
            "mtsrr0 3",
            "lwz 3,132(4)",
            "mtsrr1 3",
            "lwz 3,12(4)",
            "mfspr 4,{SPRG3}",
            "rfi",
            SPRG3 = const 275,
            MSR_DR = const 0x10,
            MSR_IR = const 0x20,
            MSR_FP = const 0x2000,
            default_exception = sym default_exception,
            CONTEXT = sym CONTEXT,
            options(noreturn)
        )
    }
}

#[naked]
#[allow(named_asm_labels)]
pub extern "C" fn recoverable_exception_handler() {
    unsafe {
        core::arch::asm!(
            ".global RECOVERABLE_HANDER_START",
            "RECOVERABLE_HANDLER_START:",
            "mtspr {SPRG3},4",
            "lis 4,{CONTEXT}@h",
            "ori 4,4,{CONTEXT}@l",
            "clrlwi 4,4,2",
            //STORE CONTEXT
            "stw 0,0(4)",
            "stw 1,4(4)",
            "stw 2,8(4)",
            "stw 3,12(4)",
            "mfspr 3,{SPRG3}",
            "stw 3,16(4)",
            "stw 5,20(4)",
            "mfsrr0 3",
            "stw 3,128(4)",
            "mfsrr1 3",
            "stw 3,132(4)",
            "mfcr 3",
            "stw 3,136(4)",
            "mflr 3",
            "stw 3,140(4)",
            "mfctr 3",
            "stw 3,144(4)",
            "mfxer 3",
            "stw 3,148(4)",
            "mfmsr 3",
            "stw 3,152(4)",
            "mfdar 3",
            "stw 3,156(4)",
            //END STORE CONTEXT
            "lis 3,default_recoverable@h",
            "ori 3,3,default_recoverable@l",
            "mtsrr0 3",
            "mfmsr 3",
            "ori 3,3,{MSR_DR}|{MSR_IR}|{MSR_FP}",
            "mtsrr1 3",
            "bl 1f",
            "1:",
            "mflr 3",
            "subi 3,3,0x88",
            "rfi",
            ".global RECOVERABLE_HANDLER_END",
            "RECOVERABLE_HANDLER_END:",
            "nop",
            "default_recoverable:",
            "lis 4,{CONTEXT}@h",
            "ori 4,4,{CONTEXT}@l",
            "stw 6,24(4)",
            "stw 7,28(4)",
            "stw 8,32(4)",
            "stw 9,36(4)",
            "stw 10,40(4)",
            "stw 11,44(4)",
            "stw 12,48(4)",
            "stw 13,52(4)",
            "stw 14,56(4)",
            "stw 15,60(4)",
            "stw 16,64(4)",
            "stw 17,68(4)",
            "stw 18,72(4)",
            "stw 19,76(4)",
            "stw 20,80(4)",
            "stw 21,84(4)",
            "stw 22,88(4)",
            "stw 23,92(4)",
            "stw 24,96(4)",
            "stw 25,100(4)",
            "stw 26,104(4)",
            "stw 27,108(4)",
            "stw 28,112(4)",
            "stw 29,116(4)",
            "stw 30,120(4)",
            "stw 31,124(4)",
            "mfmsr 5",
            "ori 5,5,{MSR_RI}",
            "mtmsr 5",
            "isync",
            "bl {default_exception}",
            "lis 4,{CONTEXT}@h",
            "ori 4,4,{CONTEXT}@l",
            "lwz 3,136(4)",
            "mtcr 3",
            "lwz 3,140(4)",
            "mtlr 3",
            "lwz 3,144(4)",
            "mtctr 3",
            "lwz 3,148(4)",
            "mtxer 3",
            "lwz 6,24(4)",
            "lwz 7,28(4)",
            "lwz 8,32(4)",
            "lwz 9,36(4)",
            "lwz 10,40(4)",
            "lwz 11,44(4)",
            "lwz 12,48(4)",
            "lwz 13,52(4)",
            "lwz 14,56(4)",
            "lwz 15,60(4)",
            "lwz 16,64(4)",
            "mfmsr 3",
            "rlwinm 3,3,0,31,29",
            "mtmsr 3",
            "isync",
            "lwz 0,0(4)",
            "lwz 2,8(4)",
            "lwz 3,128(4)",
            "mtsrr0 3",
            "lwz 3,132(4)",
            "mtsrr1 3",
            "lwz 3,12(4)",
            "mfspr 4,{SPRG3}",
            "rfi",
            SPRG3 = const 275,
            MSR_DR = const 0x10,
            MSR_IR = const 0x20,
            MSR_FP = const 0x2000,
            MSR_RI = const 0x2,
            default_exception = sym default_exception,
            CONTEXT = sym CONTEXT,
            options(noreturn)
        )
    }
}
/// # Safety
///
/// This function must be called with within the `exception_handler`
pub unsafe extern "C" fn default_exception(addr: usize, frame: *const ExceptionFrame) {
    if let Some(exception) = Exception::from_addr(0x8000_0000 + addr) {
        Exception::invoke_exception_handler(exception, frame.as_ref().unwrap()).unwrap();
    }

    //loop {}
}

pub fn decrementer_set(ticks: usize) {
    unsafe { core::arch::asm!("mtdec {ticks}", ticks = in(reg) ticks,) }
}
