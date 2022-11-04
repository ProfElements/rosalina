#![no_std]
#![no_main]
#![feature(asm_experimental_arch, alloc_error_handler)]

extern crate alloc;

use core::{alloc::Layout, fmt::Write, panic::PanicInfo};

use rosalina::{
    exception::{decrementer_set, Exception, ExceptionSystem},
    interrupts,
    os::OS,
    DOLPHIN_HLE,
};

#[no_mangle]
extern "C" fn main() -> ! {
    let _os = OS::init();

    interrupts::disable();
    ExceptionSystem::set_exception_handler(Exception::Decrementer, |_, _| {
        unsafe {
            write!(DOLPHIN_HLE, "Decrementer worked").ok();
        }
        Ok(())
    });
    decrementer_set(0xFF);
    interrupts::enable();
    unsafe {
        write!(DOLPHIN_HLE, "HELLO WORLD").ok();
    }

    loop {}
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    unsafe {
        write!(DOLPHIN_HLE, "{}", info).ok();
    }
    loop {}
}

#[alloc_error_handler]
fn alloc_handler(layout: Layout) -> ! {
    unsafe {
        write!(
            DOLPHIN_HLE,
            "Failed to allocate item with \n Size: {}\n, Align: {}\n",
            layout.size(),
            layout.align()
        )
        .ok();
    }
    panic!()
}
