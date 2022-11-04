#![no_std]
#![feature(asm_experimental_arch, asm_const, naked_functions)]
#![feature(inline_const, extern_types)]

use core::fmt::Write;

use alloc::string::ToString;

extern crate alloc;

pub mod asm_runtime;
pub mod cache;
pub mod mmio;

pub mod exception;
pub mod interrupts;
pub mod os;

#[inline(never)]
#[no_mangle]
pub extern "C" fn __write_console(_unused: u32, _str: *const u8, _size: *const u32) {}

pub struct DolphinHle;
pub static mut DOLPHIN_HLE: DolphinHle = DolphinHle;

impl Write for DolphinHle {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        __write_console(0, s.as_ptr(), &(s.len() as u32) as *const u32);
        Ok(())
    }

    fn write_fmt(mut self: &mut Self, args: core::fmt::Arguments<'_>) -> core::fmt::Result {
        if let Some(str) = args.as_str() {
            self.write_str(str)
        } else {
            self.write_str(&args.to_string())
        }
    }
}
