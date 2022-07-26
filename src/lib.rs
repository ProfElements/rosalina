#![no_std]
#![feature(naked_functions, inline_const, asm_experimental_arch, extern_types)]

extern crate alloc;

pub mod exception;
pub mod interrupts;
pub mod os;
mod rt0;

use core::arch::asm;

#[inline(never)]
#[no_mangle]
pub extern "C" fn puts(unused: u32, str: *const u8) {
    unsafe {
        asm!("/* {0} {1}*/", in(reg) unused, in(reg) str);
    }
}

#[macro_export]
macro_rules! print  {
    ($($arg:tt)*) => {
        let string = alloc::fmt::format(core::format_args!($($arg)*));
        $crate::puts(0, string.as_ptr());
    };
}

