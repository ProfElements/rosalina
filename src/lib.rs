#![no_std]
#![feature(
    asm_experimental_arch,
    asm_const,
    naked_functions,
    strict_provenance,
    inline_const,
    extern_types,
    ptr_from_ref,
    slice_as_chunks
)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(
    clippy::must_use_candidate,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::unreadable_literal
)]
//TODO: instead of disable move to 150 lines

use core::fmt::Write;

use alloc::string::ToString;
use spin::Mutex;

extern crate alloc;

pub mod arch;
pub mod asm_runtime;
pub mod cache;
pub mod clock;
pub mod config;
pub mod exception;
pub mod exi;
pub mod interrupts;
pub mod ios;
pub mod ipc;
pub mod mmio;
pub mod os;
pub mod pad;
pub mod si;
pub mod sram;
pub mod vi;
pub mod wii;
/// # Safety
///
/// Most use a valid string pointer and length must be valid and non-zero
#[inline(never)]
#[no_mangle]
pub(crate) unsafe extern "C" fn __write_console(_unused: u32, str: *const u8, size: *const u32) {
    static mut BUFFER: [u8; 2 ^ 1024] = [0u8; 2 ^ 1024];
    let string = unsafe {
        core::str::from_utf8(core::slice::from_raw_parts(
            str,
            usize::try_from(*size).unwrap(),
        ))
        .unwrap_or_default()
    };
    for (n, byte) in string.bytes().enumerate() {
        BUFFER[n] = byte;
    }
}

pub static mut DOLPHIN_HLE: Writer = Writer;

pub struct Writer;

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let len: u32 = u32::try_from(s.len()).expect("String length is longer then u32::MAX");
        unsafe {
            __write_console(
                core::ptr::from_mut(self).addr().try_into().unwrap(),
                s.as_ptr(),
                &len,
            );
        }
        Ok(())
    }

    fn write_fmt(&mut self, args: core::fmt::Arguments<'_>) -> core::fmt::Result {
        if let Some(s) = args.as_str() {
            self.write_str(s)
        } else {
            self.write_str(&args.to_string())
        }
    }
}

pub fn __print(args: core::fmt::Arguments) {
    static WRITER: Mutex<Writer> = Mutex::new(Writer);

    interrupts::disable();
    let mut writer = WRITER.lock();
    writer.write_fmt(args).unwrap();
    interrupts::enable();
}

#[macro_export]
macro_rules! print {
    ($($t:tt)*) => {
        $crate::__print(format_args!($($t)*))
    };
}
