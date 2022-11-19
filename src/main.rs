#![no_std]
#![no_main]
#![feature(asm_experimental_arch, alloc_error_handler)]

extern crate alloc;

use core::{
    alloc::Layout,
    fmt::Write,
    panic::PanicInfo,
    sync::atomic::{AtomicUsize, Ordering},
};

use rosalina::{
    exception::{decrementer_set, Exception},
    interrupts::{self, Interrupt},
    os::OS,
    DOLPHIN_HLE,
};

static RETRACE_COUNT: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
extern "C" fn main() -> ! {
    let _os = OS::init();

    interrupts::disable();
    Exception::set_exception_handler(Exception::Decrementer, |_, _| {
        unsafe {
            write!(DOLPHIN_HLE, "Decrementer worked").ok();
        }
        Ok(())
    });

    Interrupt::set_interrupt_handler(Interrupt::VideoInterface, |_| {
        RETRACE_COUNT.fetch_add(1, Ordering::Acquire);
        Ok(())
    });
    decrementer_set(0xFF);
    interrupts::enable();
    unsafe {
        write!(DOLPHIN_HLE, "HELLO WORLD").ok();
    }

    // This is required due to decrementer interrupt happening anytime within the loop
    #[allow(clippy::empty_loop)]
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
