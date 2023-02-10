#![no_std]
#![no_main]
#![feature(asm_experimental_arch, alloc_error_handler, strict_provenance)]

extern crate alloc;

use core::{alloc::Layout, panic::PanicInfo, ptr::from_exposed_addr};

use rosalina::{
    clock::Instant,
    exception::{decrementer_set, Exception},
    exi::ExternalInterface,
    interrupts,
    mmio::si::SiChannel,
    os::OS,
    pad::Pad,
    println,
    vi::{ViFramebuffer, VideoSystem},
};

#[no_mangle]
extern "C" fn main() -> ! {
    let _os = OS::init();

    interrupts::disable();
    Exception::set_exception_handler(Exception::Decrementer, |_, _| {
        println!("Decrementer worked");
        Ok(())
    });
    decrementer_set(0xFF);
    interrupts::enable();

    println!("Hello, world!");

    let mut vi = VideoSystem::new(ViFramebuffer::new(640, 480));
    let write_ptr = vi.framebuffer.data.as_mut_ptr().cast::<u16>();
    let _sram = ExternalInterface::get_sram();
    let pad = Pad::init(SiChannel::Zero).unwrap();

    'main_loop: loop {
        let status = pad.read();

        println!("Pad zero status: {status:?}");

        if status.start() {
            break 'main_loop;
        }

        let start_draw_time = Instant::now();

        for i in 0..(vi.framebuffer.width * vi.framebuffer.height) {
            unsafe {
                write_ptr.offset(i.try_into().unwrap()).write(0xff80);
            }
        }

        let end_draw_time = Instant::now();

        println!(
            "Draw time: {} milliseconds",
            (end_draw_time - start_draw_time).millisecs()
        );

        println!(
            "Monotick clock: {} seconds | RTC clock: {} seconds",
            Instant::now().secs(),
            ExternalInterface::get_rtc()
        );

        vi.wait_for_retrace();
    }
    unsafe { abort() }
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {
        core::hint::spin_loop();
    }
}

#[alloc_error_handler]
fn alloc_handler(layout: Layout) -> ! {
    println!(
        "Failed to allocate item with \n Size: {}\n, Align: {}\n",
        layout.size(),
        layout.align()
    );
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
        println!("Found stub: {str}");
        let func = unsafe {
            core::mem::transmute::<*const (), extern "C" fn() -> !>(from_exposed_addr(0x80001800))
        };
        func()
    }
    panic!()
}
