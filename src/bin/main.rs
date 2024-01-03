#![no_std]
#![feature(
    start,
    asm_experimental_arch,
    alloc_error_handler,
    strict_provenance,
    exposed_provenance
)]
#![cfg_attr(not(miri), no_main)]

extern crate alloc;

use core::{alloc::Layout, panic::PanicInfo, ptr::from_exposed_addr};

use rosalina::{
    clock::Instant,
    exception::{decrementer_set, Exception},
    exi::ExternalInterface,
    gfx,
    gx::Fifo,
    interrupts, isfs,
    mmio::si::SiChannel,
    os::OS,
    pad::Pad,
    println,
    vi::{ViFramebuffer, VideoSystem},
    SDCard,
};

#[cfg(miri)]
#[start]
fn start(_: isize, _: *const *const u8) -> isize {
    main();
    0
}

#[no_mangle]
extern "C" fn main() -> ! {
    let _os = OS::init();
    let pad = Pad::init(SiChannel::Zero).unwrap();

    interrupts::disable();
    Exception::set_exception_handler(Exception::Decrementer, |_, _| {
        println!("Decrementer worked");
        Ok(())
    });
    decrementer_set(0xFF);
    interrupts::enable();

    let mut fifo = Fifo::new().init_buffer(32 * 1024).unwrap();
    fifo.set_as_cpu_fifo();
    fifo.set_as_gpu_fifo();
    if fifo.link_cpu_gpu() {
        interrupts::disable();
        fifo.set_interrupts();
        interrupts::enable();
        gfx::enable_write_gather_pipe();
        println!("GXFIFO linked!");
    }

    unsafe {
        (0xCC00_8000 as *mut u8).write_volatile(0x61);
        (0xCC00_8000 as *mut u32).write_volatile(0x4800_FEED);
        (0xCC00_8000 as *mut u8).write_volatile(0x61);
        (0xCC00_8000 as *mut u32).write_volatile(0x4700_DEAD);

        for _ in 0..8 {
            (0xCC00_8000 as *mut u32).write_volatile(0x0);
        }
    };

    println!("Hello, world!");

    let mut vi = VideoSystem::new(ViFramebuffer::new(640, 480));
    let write_ptr = vi.framebuffer.data.as_mut_ptr().cast::<u16>();
    let _sram = ExternalInterface::get_sram();

    fifo.set_copy_clear([255, 255, 255, 255], 0x00_FF_FF_FF);

    fifo.set_viewport(
        0.0,
        0.0,
        vi.framebuffer.width as f32,
        vi.framebuffer.height as f32,
        0.0,
        1.0,
    );

    fifo.set_y_scale(1.);

    fifo.set_scissor(
        0,
        0,
        vi.framebuffer.width.try_into().unwrap(),
        vi.framebuffer.height.try_into().unwrap(),
    );

    fifo.set_copy_display_source(0, 0, vi.framebuffer.width, vi.framebuffer.height);
    fifo.set_copy_display_destination(&vi.framebuffer);

    fifo.set_copy_filter_default();
    // fifo.set_su_lpsize(6, 6, 0, 0, false);
    //fifo.set_gen_mode(1, 1, false, 1, 0, 0, 0);
    let mut sd_card = SDCard::new().expect("Couldn't open sd_card");
    let sectors = [0u8; 512];

    let _resp = sd_card.read_sectors(0, &mut [sectors]).unwrap();
    let _resp = sd_card.num_bytes().unwrap();

    println!("Sector read: {:?}", sectors);
    println!(
        "Resp: {:X}, {:X}, {:X}, {:X}",
        _resp[0], _resp[1], _resp[2], _resp[3]
    );

    let sysconf_data = isfs::read("/shared2/sys/SYSCONF").unwrap();

    println!("{sysconf_data:?}");

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
    main();
}
