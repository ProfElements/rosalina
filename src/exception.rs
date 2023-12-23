use core::fmt::Display;
use core::{arch::asm, fmt::Write};

use alloc::boxed::Box;
use spin::RwLock;

use crate::cache::{dc_flush_range_no_sync, ic_invalidate_range};
use crate::interrupts::{self, interrupt_handler};

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
}

impl ExceptionFrame {
    pub const fn new() -> Self {
        Self {
            srr0: 0,
            srr1: 0,
            gprs: [0; 32],
            cr: 0,
            lr: 0,
            ctr: 0,
            xer: 0,
            msr: 0,
            dar: 0,
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

pub fn decrementer_set(ticks: usize) {
    unsafe { core::arch::asm!("mtdec {ticks}", ticks = in(reg) ticks,) }
}

#[repr(C)]
pub struct Context {
    gprs: GeneralPurposeRegisters,
    sprs: SpecialPurposeRegisters,
}

impl Context {
    pub const fn new() -> Self {
        Self {
            gprs: GeneralPurposeRegisters::new(),
            sprs: SpecialPurposeRegisters::new(),
        }
    }
}

#[derive(Default)]
pub struct GeneralPurposeRegisters {
    pub gpr0: usize,
    pub gpr1: usize,
    pub gpr2: usize,
    pub gpr3: usize,
    pub gpr4: usize,
    pub gpr5: usize,
    pub gpr6: usize,
    pub gpr7: usize,
    pub gpr8: usize,
    pub gpr9: usize,
    pub gpr10: usize,
    pub gpr11: usize,
    pub gpr12: usize,
    pub gpr13: usize,
    pub gpr14: usize,
    pub gpr15: usize,
    pub gpr16: usize,
    pub gpr17: usize,
    pub gpr18: usize,
    pub gpr19: usize,
    pub gpr20: usize,
    pub gpr21: usize,
    pub gpr22: usize,
    pub gpr23: usize,
    pub gpr24: usize,
    pub gpr25: usize,
    pub gpr26: usize,
    pub gpr27: usize,
    pub gpr28: usize,
    pub gpr29: usize,
    pub gpr30: usize,
    pub gpr31: usize,
}

impl GeneralPurposeRegisters {
    pub const fn new() -> Self {
        Self {
            gpr0: 0,
            gpr1: 0,
            gpr2: 0,
            gpr3: 0,
            gpr4: 0,
            gpr5: 0,
            gpr6: 0,
            gpr7: 0,
            gpr8: 0,
            gpr9: 0,
            gpr10: 0,
            gpr11: 0,
            gpr12: 0,
            gpr13: 0,
            gpr14: 0,
            gpr15: 0,
            gpr16: 0,
            gpr17: 0,
            gpr18: 0,
            gpr19: 0,
            gpr20: 0,
            gpr21: 0,
            gpr22: 0,
            gpr23: 0,
            gpr24: 0,
            gpr25: 0,
            gpr26: 0,
            gpr27: 0,
            gpr28: 0,
            gpr29: 0,
            gpr30: 0,
            gpr31: 0,
        }
    }
}

#[derive(Default)]
pub struct SpecialPurposeRegisters {
    pub srr0: usize,
    pub srr1: usize,
    pub cr: usize,
    pub lr: usize,
    pub ctr: usize,
    pub xer: usize,
    pub msr: usize,
    pub dar: usize,
}

impl SpecialPurposeRegisters {
    pub const fn new() -> Self {
        Self {
            srr0: 0,
            srr1: 0,
            cr: 0,
            lr: 0,
            ctr: 0,
            xer: 0,
            msr: 0,
            dar: 0,
        }
    }
}

static EXCEPTION_CONTEXT: Context = Context::new();

/// # Safety
///
/// Must be called by an the calling of an exceptions
/// **DO NOT CALL THIS DIRECTLY EVER**
#[no_mangle]
#[naked]
#[allow(named_asm_labels)]
pub unsafe extern "C" fn exception_handler_shim() -> ! {
    core::arch::asm!(
        ".global EXCEPTION_HANDLER_START",
        "EXCEPTION_HANDLER_START:",
        "mtsprg 3,4",
        "lis 4,{CONTEXT}@h",
        "ori 4,4,{CONTEXT}@l",
        "clrlwi 4,4,2",
        //Store general purpose usable
        "stw 0,0(4)",
        "stw 1,4(4)",
        "stw 2,8(4)",
        "stw 3,12(4)",
        //Move sprg3 to 3 which has reg 4
        "mfsprg 3,3",
        "stw 3,16(4)",

        "stw 5,20(4)",
        // End store general purpose usable
        // Start store special purpose
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
        "stw 3, 148(4)",
        "mfmsr 3",
        "stw 3, 152(4)",
        "mfdar 3",
        "stw 3,156(4)",
        //End store store special purpose
        "lis 3,{EXCEPTION_HANDLER}@h",
        "ori 3,3,{EXCEPTION_HANDLER}@l",
        "mtsrr0 3",
        "mfmsr 3",
        "ori 3,3,{FLAGS}",
        "mtsrr1 3",
        "bl 1f",
        "1:",
        "mflr 3",
        "subi 3,3,0x88",
        "rfi",
        ".global EXCEPTION_HANDLER_END",
        "EXCEPTION_HANDLER_END:",
        "nop",
        CONTEXT = sym EXCEPTION_CONTEXT,
        EXCEPTION_HANDLER = sym de_exception_handler,
        FLAGS = const handler_flags(),
        options(noreturn)
    )
}

const fn handler_flags() -> usize {
    const MSR_DR: usize = 0x10;
    const MSR_IR: usize = 0x20;
    const MSR_FP: usize = 0x2000;
    MSR_DR | MSR_IR | MSR_FP
}

/// # Safety
///
/// MUST BE CALLED FORM `exception_handler_shim`
#[no_mangle]
#[naked]
pub unsafe extern "C" fn de_exception_handler() -> ! {
    core::arch::asm!(
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
        "isync",
        "sync",
        "bl {DEFAULT_EXCEPTION_HANDLER}",
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
        "mfsprg 4,3",
        "rfi",
       DEFAULT_EXCEPTION_HANDLER = sym def_exception_handler,
        CONTEXT = sym EXCEPTION_CONTEXT,
        options(noreturn)
    )
}

/// # Safety
///
/// Must be provided a valid `exception_addr`, and a valid and nonnull Context
pub unsafe extern "C" fn def_exception_handler(exception_addr: usize, context: *const Context) {
    interrupts::disable();
    if let Some(exception) = Exception::from_addr(exception_addr + 0x8000_0000) {
        Exception::invoke_exception_handler(
            exception,
            context.cast::<ExceptionFrame>().as_ref().unwrap(),
        )
        .unwrap();
        return;
    }
    interrupts::enable();
    core::hint::unreachable_unchecked();
}
