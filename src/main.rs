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
    exception::{decrementer_set, Exception, ExceptionFrame},
    interrupts::{self, interrupt_handler, Interrupt},
    mmio::{
        pi::{InterruptCause, InterruptState},
        vi::DisplayInterrupt,
    },
    os::OS,
    vi::{ViFramebuffer, VideoSystem},
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
        let _retcnt = RETRACE_COUNT.fetch_add(1, Ordering::AcqRel);
        /*unsafe {
            write!(DOLPHIN_HLE, "{}", retcnt).ok();
        }*/

        if DisplayInterrupt::read_zero().status() == InterruptState::Happened {
            DisplayInterrupt::read_zero()
                .with_status(InterruptState::Idle)
                .write_zero();
        }

        if DisplayInterrupt::read_one().status() == InterruptState::Happened {
            DisplayInterrupt::read_one()
                .with_status(InterruptState::Idle)
                .write_one();
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

    // This is required due to decrementer interrupt happening anytime within the loop
    loop {
        //VERY TEMPORARY

        unsafe {
            write!(
                DOLPHIN_HLE,
                "{:?}",
                InterruptCause::read().video_interface()
            )
            .ok();

            write!(
                DOLPHIN_HLE,
                "pre-retrace: {:?}, post-retrace: {:?}",
                DisplayInterrupt::read_zero().status(),
                DisplayInterrupt::read_one().status()
            )
            .ok();
        };
        for i in 0..(vi.framebuffer.width * vi.framebuffer.height) {
            unsafe {
                write_ptr.offset(i.try_into().unwrap()).write(0xff80);
            }
        }

        let retcnt = RETRACE_COUNT.load(Ordering::Acquire);
        unsafe { write!(DOLPHIN_HLE, "Retrace count: {}", retcnt).ok() };
        //interrupt_handler(0x80000700, &ExceptionFrame::default()).ok();
        while RETRACE_COUNT.load(Ordering::Acquire) == retcnt {}
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
