#![no_std]
#![no_main]
#![feature(asm_experimental_arch, alloc_error_handler)]

extern crate alloc;

use core::{alloc::Layout, fmt::Write, panic::PanicInfo};

use rosalina::{
    clock::Instant,
    exception::{decrementer_set, Exception},
    exi::ExternalInterface,
    interrupts,
    os::OS,
    vi::{ViFramebuffer, VideoSystem},
    DOLPHIN_HLE,
};

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
    decrementer_set(0xFF);
    interrupts::enable();

    unsafe {
        write!(DOLPHIN_HLE, "HELLO WORLD").ok();
    }

    let mut vi = VideoSystem::new(ViFramebuffer::new(640, 480));
    let write_ptr = vi.framebuffer.data.as_mut_ptr().cast::<u16>();
    let _sram = ExternalInterface::get_sram();
    loop {
        let time = Instant::now().ticks;

        for i in 0..(vi.framebuffer.width * vi.framebuffer.height) {
            unsafe {
                write_ptr.offset(i.try_into().unwrap()).write(0xff80);
            }
        }

        let diff = Instant::now().ticks.wrapping_sub(time);
        unsafe {
            write!(
                DOLPHIN_HLE,
                "Rendering takes {} millisecs",
                Instant { ticks: diff }.millisecs()
            )
            .ok();
            write!(DOLPHIN_HLE, "Monotick clock: {}", Instant::now().secs()).ok();
            write!(DOLPHIN_HLE, "RTC clock: {}", ExternalInterface::get_rtc()).ok();
        };

        vi.wait_for_retrace();
    }
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
