#![no_std]
#![no_main]
#![feature(asm_experimental_arch, alloc_error_handler, strict_provenance)]

extern crate alloc;

use core::{alloc::Layout, fmt::Write, panic::PanicInfo, ptr::from_exposed_addr};

use rosalina::{
    clock::Instant,
    exception::{decrementer_set, Exception},
    exi::ExternalInterface,
    interrupts,
    mmio::si::SiChannel,
    os::OS,
    pad::Pad,
    si::SerialInterface,
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
    let pad = Pad::init(SiChannel::Zero).unwrap();

    loop {
        let time = Instant::now().ticks;
        let status = pad.read();
        unsafe { write!(DOLPHIN_HLE, "{status:?}").unwrap() }
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
    //unsafe { abort() }
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    unsafe {
        write!(DOLPHIN_HLE, "{info}").ok();
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

/// # Safety
///
/// Must be called when you use the homebrew loader and its must be setup tro have a stubb
pub unsafe extern "C" fn abort() -> ! {
    let str = core::str::from_utf8(core::slice::from_raw_parts(
        from_exposed_addr(0x8000_1804),
        8,
    ))
    .unwrap();

    if str == "STUBHAXX" {
        write!(DOLPHIN_HLE, "Found stub {str}").ok();
        let func = unsafe {
            core::mem::transmute::<*const (), extern "C" fn() -> !>(from_exposed_addr(0x80001800))
        };
        func()
    }
    panic!()
}
