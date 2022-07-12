#![no_std]
#![no_main]
#![feature(asm_experimental_arch, alloc_error_handler)]

extern crate alloc;

use core::{alloc::Layout, panic::PanicInfo};

use rosalina::{os::OS, print};

#[no_mangle]
extern "C" fn main() -> ! {
    let _os = OS::init();

    loop {}
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    print!("{}", info);
    loop {}
}

#[alloc_error_handler]
fn alloc_handler(layout: Layout) -> ! {
    print!(
        "Failed to allocate item with \n Size: {}\n, Align: {}\n",
        layout.size(),
        layout.align()
    );
    loop {}
}
