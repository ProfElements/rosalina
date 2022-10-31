#![no_std]
#![feature(asm_experimental_arch, asm_const, naked_functions)]
#![feature(inline_const, extern_types)]
extern crate alloc;

pub mod asm_runtime;
pub mod cache;
pub mod mmio;

pub mod exception;
pub mod interrupts;
pub mod os;

#[inline(never)]
#[no_mangle]
pub extern "C" fn __write_console(_unused: u32, str: *const u8, size: *const u32) {
    unsafe {
        core::arch::asm!("/* {0} {1}*/", in(reg) str, in(reg) size);
    }
}

#[macro_export]
macro_rules! print  {
    ($($arg:tt)*) => {
        let string = alloc::fmt::format(core::format_args!($($arg)*));
        $crate::__write_console(0, string.as_ptr(), &(string.len() as u32) as *const u32);
    };
}
