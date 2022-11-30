use core::{
    alloc::Layout,
    mem,
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::{alloc::alloc, boxed::Box};

use crate::{
    interrupts::Interrupt,
    mmio::{
        pi::{InterruptMask, InterruptState, Mask},
        vi::{
            BurstBlankingInterval, Clock, DisplayConfig, DisplayInterlacedMode, DisplayInterrupt,
            Enabled, FieldVerticalTiming, FilterCoeffTableOne, FilterCoeffTableZero, Framebuffer,
            HorizontalScale, HorizontalSteppingWidth, HorizontalTimingOne, HorizontalTimingZero,
            VerticalTiming, VideoClock, VideoFormat,
        },
    },
};

pub struct ViFramebuffer {
    pub width: usize,
    pub height: usize,
    pub data: Pin<Box<[u8]>>,
}

impl ViFramebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let slice = unsafe {
            let ptr =
                alloc(Layout::from_size_align(width * height * mem::size_of::<u16>(), 32).unwrap());
            Box::from_raw(ptr.cast::<[u8; 1]>())
        };
        Self {
            width,
            height,
            data: Pin::new(slice),
        }
    }
}

pub struct VideoSystem {
    //TODO: REMOVE THIS DUMMY STUFF
    pub framebuffer: ViFramebuffer,
}

static RETRACE_COUNT: AtomicUsize = AtomicUsize::new(0);
impl VideoSystem {
    pub fn new(framebuffer: ViFramebuffer) -> Self {
        VerticalTiming::new()
            .with_active_video_lines((framebuffer.height / 2).try_into().unwrap())
            .with_equalizaion_pulse(6)
            .write();

        DisplayConfig::new()
            .with_video_format(VideoFormat::Pal)
            .with_display_interlaced_mode(DisplayInterlacedMode::Interlaced)
            .with_enabled(Enabled::Enabled)
            .write();

        HorizontalTimingZero::new()
            .with_color_burst_start(71)
            .with_color_burst_end(105)
            .with_halfline_width(429)
            .write();

        HorizontalTimingOne::new()
            .with_horizontal_blanking_start(373)
            .with_horizontal_blanking_end(162)
            .with_horizontal_sync_width(64)
            .write();

        FieldVerticalTiming::new()
            .with_pre_blanking(3)
            .with_post_blanking(24)
            .write_odd();

        FieldVerticalTiming::new()
            .with_pre_blanking(2)
            .with_post_blanking(25)
            .write_even();

        BurstBlankingInterval::new()
            .with_burst_start(12)
            .with_burst_end(520)
            .with_burst_start_two(12)
            .with_burst_end_two(520)
            .write_odd();

        BurstBlankingInterval::new()
            .with_burst_start(13)
            .with_burst_end(519)
            .with_burst_start_two(13)
            .with_burst_end_two(519)
            .write_even();

        Framebuffer::new()
            .with_addr(u32::try_from(framebuffer.data.as_ptr().addr()).unwrap() - 0x8000_0000u32)
            .with_horizontal_offset(0)
            .write_top_left();

        Framebuffer::new()
            .with_addr(
                u32::try_from(framebuffer.data.as_ptr().addr()).unwrap() - 0x8000_0000
                    + u32::try_from(framebuffer.width * 2).unwrap(),
            )
            .with_horizontal_offset(0)
            .write_bottom_left();

        DisplayInterrupt::new()
            .with_vertical_pos(263)
            .with_horizontal_pos(430)
            .with_enable(Enabled::Enabled)
            .write_zero();

        DisplayInterrupt::new()
            .with_vertical_pos(1)
            .with_horizontal_pos(1)
            .with_enable(Enabled::Enabled)
            .write_one();

        HorizontalSteppingWidth::from(0x2850).write();

        HorizontalScale::from(0x0100).write();

        FilterCoeffTableZero::from(0x1AE771F0).write_zero();
        FilterCoeffTableZero::from(0x0DB4A574).write_one();
        FilterCoeffTableZero::from(0x00C1188E).write_two();

        FilterCoeffTableOne::from(0xC4C0CBE2).write_three();
        FilterCoeffTableOne::from(0xFCECDECF).write_four();
        FilterCoeffTableOne::from(0x13130F08).write_five();
        FilterCoeffTableOne::from(0x00080C0F).write_six();

        VideoClock::new()
            .with_clock(Clock::TwentySevenMegahertz)
            .write();

        Interrupt::set_interrupt_handler(Interrupt::VideoInterface, |_| {
            RETRACE_COUNT.fetch_add(1, Ordering::Relaxed);

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

            if DisplayInterrupt::read_two().status() == InterruptState::Happened {
                DisplayInterrupt::read_two()
                    .with_status(InterruptState::Idle)
                    .write_two();
            }

            if DisplayInterrupt::read_three().status() == InterruptState::Happened {
                DisplayInterrupt::read_three()
                    .with_status(InterruptState::Idle)
                    .write_three();
            }

            Ok(())
        });

        InterruptMask::read()
            .with_video_interface(Mask::Enabled)
            .write();

        Self { framebuffer }
    }

    pub fn wait_for_retrace(&self) {
        let retcnt = RETRACE_COUNT.load(Ordering::Relaxed);
        while RETRACE_COUNT.load(Ordering::Relaxed) == retcnt {}
    }
}
