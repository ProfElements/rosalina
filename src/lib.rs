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

pub struct DolphinHle;
pub static mut DOLPHIN_HLE: DolphinHle = DolphinHle;

impl Write for DolphinHle {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let len = u32::try_from(s.len()).unwrap();
        unsafe {
            __write_console(0, s.as_ptr(), &len);
        }
        Ok(())
    }

    fn write_fmt(&mut self, args: core::fmt::Arguments<'_>) -> core::fmt::Result {
        if let Some(str) = args.as_str() {
            self.write_str(str)
        } else {
            self.write_str(&args.to_string())
        }
    }
}
