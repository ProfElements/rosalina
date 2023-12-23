use core::{alloc::Layout, num::NonZeroUsize};

use alloc::vec::Vec;
use bit_field::BitField;

use crate::{
    interrupts::Interrupt,
    mmio::{
        cp::{
            CommandClear, CommandControl, CP_FIFO_BASE_HI, CP_FIFO_BASE_LO, CP_FIFO_END_HI,
            CP_FIFO_END_LO, CP_FIFO_HIGH_MARK_HI, CP_FIFO_HIGH_MARK_LO, CP_FIFO_LO_MARK_HI,
            CP_FIFO_LO_MARK_LO, CP_FIFO_READ_PTR_HI, CP_FIFO_READ_PTR_LO,
            CP_FIFO_READ_WRITE_DST_HI, CP_FIFO_READ_WRITE_DST_LO, CP_FIFO_WRITE_PTR_HI,
            CP_FIFO_WRITE_PTR_LO,
        },
        pi::{InterruptMask, InterruptState, Mask, FIFO_BASE, FIFO_END, FIFO_WRITE_PTR},
        vi::Enabled,
    },
    utils::WriteGatherPipe,
    vi::ViFramebuffer,
};

pub struct Fifo {
    buf_start: *mut u8,
    buf_end: *mut u8,
    size: u32,
    hi_mark: u32,
    lo_mark: u32,
    read_ptr: *mut u8,
    write_ptr: *mut u8,
    read_write_distance: u32,
    cpu_ready: bool,
    gpu_ready: bool,
    _padding: [u8; 93],
}

#[derive(Debug)]
pub enum Error {
    WrongLayout,
}

impl Fifo {
    pub fn new() -> Self {
        // Safety: an all zero bit pattern is valid for a fifo.
        unsafe { core::mem::zeroed() }
    }
    /// # Errors
    /// `WrongLayout`:  Somehow the layout provided is wrong, this should never happen
    pub fn init_buffer(mut self, buffer_size: u32) -> Result<Self, Error> {
        // All wii examples align fifo buffer to 32 bytes. Will do the same

        let buffer_size = usize::try_from(buffer_size).unwrap();

        let Ok(layout) = Layout::from_size_align(buffer_size, 32) else {
            return Err(Error::WrongLayout);
        };

        let buffer_ptr = unsafe { alloc::alloc::alloc(layout) };

        self.buf_start = buffer_ptr;
        self.buf_end = unsafe { buffer_ptr.add(buffer_size - 4) };

        let buffer_size = u32::try_from(buffer_size).unwrap();
        self.size = buffer_size;
        self.read_write_distance = 0;

        self.hi_mark = buffer_size - (16 * 1024);
        self.lo_mark = (buffer_size >> 1) & 0x7fffffe0;

        let read_write_distance = u32::try_from(buffer_ptr.addr() - buffer_ptr.addr()).unwrap();
        self.read_ptr = buffer_ptr;
        self.write_ptr = buffer_ptr;
        self.read_write_distance = read_write_distance;

        Ok(self)
    }

    pub fn set_as_cpu_fifo(&mut self) {
        self.cpu_ready = true;

        CommandControl::read()
            .with_fifo_underflow_interrupt(InterruptState::Idle)
            .with_fifo_overflow_interrupt(InterruptState::Idle)
            .write();

        FIFO_BASE.write(self.buf_start.map_addr(|addr| addr & !0xC000_0000).addr());

        FIFO_END.write(self.buf_end.map_addr(|addr| addr & !0xC000_0000).addr());

        FIFO_WRITE_PTR.write(self.write_ptr.addr() & 0x1FFFFFE0);

        sync();
    }

    pub fn set_as_gpu_fifo(&mut self) {
        CommandControl::read()
            .with_gp_fifo_read_enable(Enabled::Disabled)
            .with_fifo_overflow_interrupt(InterruptState::Idle)
            .with_fifo_underflow_interrupt(InterruptState::Idle)
            .write();

        self.gpu_ready = true;

        let buf_start_addr = u32::try_from(self.buf_start.addr()).unwrap();
        let buf_end_addr = u32::try_from(self.buf_end.addr()).unwrap();
        let high = self.hi_mark;
        let lo = self.lo_mark;
        let buf_read_addr = u32::try_from(self.read_ptr.addr()).unwrap();
        let buf_write_addr = u32::try_from(self.write_ptr.addr()).unwrap();
        let read_write_dst = self.read_write_distance;

        CP_FIFO_BASE_LO.write(buf_start_addr.get_bits(0..=15).try_into().unwrap());
        CP_FIFO_BASE_HI.write(buf_start_addr.get_bits(16..=31).try_into().unwrap());

        CP_FIFO_END_LO.write(buf_end_addr.get_bits(0..=15).try_into().unwrap());
        CP_FIFO_END_HI.write(buf_end_addr.get_bits(16..=31).try_into().unwrap());
        CP_FIFO_HIGH_MARK_LO.write(high.get_bits(0..=15).try_into().unwrap());
        CP_FIFO_HIGH_MARK_HI.write(high.get_bits(16..=31).try_into().unwrap());
        CP_FIFO_LO_MARK_LO.write(lo.get_bits(0..=15).try_into().unwrap());
        CP_FIFO_LO_MARK_HI.write(lo.get_bits(16..=31).try_into().unwrap());
        CP_FIFO_READ_WRITE_DST_LO.write(read_write_dst.get_bits(0..=15).try_into().unwrap());
        CP_FIFO_READ_WRITE_DST_HI.write(read_write_dst.get_bits(16..=31).try_into().unwrap());
        CP_FIFO_WRITE_PTR_LO.write(buf_write_addr.get_bits(0..=15).try_into().unwrap());
        CP_FIFO_WRITE_PTR_HI.write(buf_write_addr.get_bits(16..=31).try_into().unwrap());
        CP_FIFO_READ_PTR_LO.write(buf_read_addr.get_bits(0..=15).try_into().unwrap());
        CP_FIFO_READ_PTR_HI.write(buf_read_addr.get_bits(16..=31).try_into().unwrap());

        CommandClear::read()
            .with_fifo_overflow(true)
            .with_fifo_underflow(true)
            .write();

        CommandControl::read()
            .with_gp_fifo_read_enable(Enabled::Enabled)
            .write();

        sync();
    }

    pub fn link_cpu_gpu(&self) -> bool {
        if self.cpu_ready && self.gpu_ready {
            CommandControl::read()
                .with_fifo_underflow_interrupt(InterruptState::Idle)
                .with_fifo_overflow_interrupt(InterruptState::Happened)
                .with_fifo_link_enable(Enabled::Enabled)
                .write();

            sync();
        }
        self.cpu_ready && self.gpu_ready
    }

    pub fn set_interrupts(&self) {
        Interrupt::set_interrupt_handler(Interrupt::PixelEngineToken, |_| {
            let status = 0xCC00_100A as *mut u16;
            let mut val = unsafe { status.read_volatile() };
            if val.get_bit(2) {
                val.set_bit(2, true);
                unsafe {
                    status.write_volatile(val);
                }
            }

            Ok(())
        });

        Interrupt::set_interrupt_handler(Interrupt::PixelEngineFinish, |_| {
            let status = 0xCC00_100A as *mut u16;
            let mut val = unsafe { status.read_volatile() };

            if val.get_bit(3) {
                val.set_bit(3, true);
                unsafe {
                    status.write_volatile(val);
                }
            }

            Ok(())
        });

        let status = 0xCC00_100A as *mut u16;
        let mut val = unsafe { status.read_volatile() };

        val.set_bit(0, true);
        val.set_bit(1, true);
        unsafe {
            status.write_volatile(val);
        }

        InterruptMask::read()
            .with_pixel_engine_token(Mask::Enabled)
            .with_pixel_engine_finish(Mask::Enabled)
            .write();
    }

    pub fn set_copy_clear(&mut self, color: [u8; 4], z_value: u32) {
        let [r, g, b, a] = color;
        let z_value = z_value.min(0x00_FF_FF_FF);

        let mut ra = 0u16;
        ra.set_bits(0..8, r.into()).set_bits(8..16, a.into());
        let mut gb = 0u16;
        gb.set_bits(0..8, b.into()).set_bits(8..16, g.into());

        let pipe_u8 = 0xCC00_8000 as *mut u8;
        let pipe_u32 = 0xCC00_8000 as *mut u32;

        unsafe {
            pipe_u8.write_volatile(0x61);
            pipe_u32.write_volatile(0x4F00_0000u32 | u32::from(ra));
            pipe_u8.write_volatile(0x61);
            pipe_u32.write_volatile(0x5000_0000u32 | u32::from(gb));
            pipe_u8.write_volatile(0x61);
            pipe_u32.write_volatile(0x5100_0000u32 | z_value);
        }
    }

    pub fn set_viewport(
        &mut self,
        x_origin: f32,
        y_origin: f32,
        width: f32,
        height: f32,
        near_plane: f32,
        far_plane: f32,
    ) {
        const X_FACTOR: f32 = 0.5;
        const Y_FACTOR: f32 = 342.0;
        const Z_FACTOR: f32 = 16777215.0;

        let x_0 = width * X_FACTOR;
        let y_0 = (-height) * X_FACTOR;
        let x_1 = x_origin + x_0 + Y_FACTOR;
        let y_1 = y_origin + (height * X_FACTOR) + Y_FACTOR;
        let near = Z_FACTOR * near_plane;
        let far = Z_FACTOR * far_plane;
        let z = far - near;

        let pipe_u8 = 0xCC00_8000 as *mut u8;

        let pipe_u32 = 0xCC00_8000 as *mut u32;
        let pipe_f32 = 0xCC00_8000 as *mut f32;

        unsafe {
            let array = [x_0, y_0, z, x_1, y_1, far];
            for n in 0..6 {
                pipe_u8.write_volatile(0x10);
                pipe_u32.write_volatile(0x101a + n);
                pipe_f32.write_volatile(array[n as usize]);
            }
        }
    }

    pub fn set_scissor(&mut self, x_origin: u32, y_origin: u32, width: u32, height: u32) {
        let xo = (x_origin + 0x156).min(0x7FF);
        let yo = (y_origin + 0x156).min(0x7FF);
        let nwd = xo + (width - 1).min(0x7FF);
        let nht = yo + (height - 1).min(0xFFF);

        let mut top_left = 0u32;
        let mut bottom_right = 0u32;

        top_left.set_bits(0..12, yo).set_bits(12..22, xo);
        bottom_right.set_bits(0..12, nht).set_bits(12..22, nwd);

        let pipe_u8 = 0xCC00_8000 as *mut u8;
        let pipe_u32 = 0xCC00_8000 as *mut u32;

        unsafe {
            pipe_u8.write_volatile(0x61);
            pipe_u32.write_volatile(0x2000_0000 | top_left);
            pipe_u8.write_volatile(0x61);
            pipe_u32.write_volatile(0x2100_0000 | bottom_right);
        }
    }

    pub fn set_copy_display_source(
        &mut self,
        left: usize,
        top: usize,
        width: usize,
        height: usize,
    ) {
        let top_left = left << 10 | top;
        let width_height = (width - 1) << 10 | (height - 1);

        let pipe_u8 = 0xCC00_8000 as *mut u8;
        let pipe_u32 = 0xCC00_8000 as *mut u32;

        unsafe {
            pipe_u8.write_volatile(0x61);
            pipe_u32.write_volatile(0x4900_0000u32 | u32::try_from(top_left).unwrap());
            pipe_u8.write_volatile(0x61);
            pipe_u32.write_volatile(0x4A00_0000 | u32::try_from(width_height).unwrap());
        }
    }

    pub fn set_copy_display_destination(&mut self, framebuffer: &ViFramebuffer) {
        let mut dest = bitfrob::u32_with_value(24, 31, 0u32, 0x4b);
        dest = bitfrob::u32_with_value(
            0,
            24,
            dest,
            framebuffer
                .data
                .as_ptr()
                .map_addr(|addr| addr - 0x8000_0000)
                .addr()
                .try_into()
                .unwrap(),
        );

        unsafe {
            let pipe_u8 = 0xCC00_8000 as *mut u8;
            let pipe_u32 = 0xCC00_8000 as *mut u32;

            pipe_u8.write_volatile(0x61);
            pipe_u32.write_volatile(dest);
        }
    }

    pub fn set_copy_display_distance(&mut self, width: usize) {
        unsafe {
            (0xCC00_8000 as *mut u8).write_volatile(0x61);
            (0xCC00_8000 as *mut u32)
                .write_volatile(0x4D00_0000 | u32::try_from(width & 0x3FF).unwrap());
        }
    }

    pub fn set_copy_display_control(
        &mut self,
        left_right_clamp: u8,
        top_bottom_clamp: u8,
        gamma: u8,
        needs_y_scale: bool,
        clear_framebuffer: bool,
        frame_2_field: u8,
        dest: u8,
    ) {
    }

    pub fn set_y_scale(&mut self, y_scale: f32) {
        let val = 256.0 / y_scale;

        let pipe_u8 = 0xCC00_8000 as *mut u8;
        let pipe_u32 = 0xCC00_8000 as *mut u32;

        unsafe {
            pipe_u8.write_volatile(0x61);
            pipe_u32.write_volatile(0x4E00_0000u32 | val as u32);
        }
    }

    pub fn set_copy_filter_default(&mut self) {
        let pipe_u8 = 0xCC00_8000 as *mut u8;
        let pipe_u32 = 0xCC00_8000 as *mut u32;

        unsafe {
            for i in 1..=4 {
                pipe_u8.write_volatile(0x61);
                pipe_u32.write_volatile(i << 24 | 0x66_66_66);
            }
        }

        unsafe {
            pipe_u8.write_volatile(0x61);
            pipe_u32.write_volatile(0x53595000);
            pipe_u8.write_volatile(0x61);
            pipe_u32.write_volatile(0x54000015);
        }
    }

    pub fn set_su_lpsize(
        &mut self,
        line_size: u8,
        point_size: u8,
        line_offset: u8,
        point_offset: u8,
        line_aspect_ratio: bool,
    ) {
        let mut su_lpsize = bitfrob::u32_with_value(0, 7, 0u32, line_size.into());
        su_lpsize = bitfrob::u32_with_value(8, 15, su_lpsize, point_size.into());
        su_lpsize = bitfrob::u32_with_value(16, 18, su_lpsize, line_offset.into());
        su_lpsize = bitfrob::u32_with_value(19, 21, su_lpsize, point_offset.into());
        su_lpsize = bitfrob::u32_with_bit(22, su_lpsize, line_aspect_ratio.into());
        su_lpsize = bitfrob::u32_with_value(24, 31, su_lpsize, 0x22);

        unsafe {
            let pipe_u8 = 0xCC00_8000 as *mut u8;
            let pipe_u32 = 0xCC00_8000 as *mut u32;

            pipe_u8.write_volatile(0x61);
            pipe_u32.write_volatile(su_lpsize);
        }
    }

    pub fn set_gen_mode(
        &mut self,
        num_tex_coords: u8,
        num_colors: u8,
        ms_en: bool,
        num_tex_stages: u8,
        cull_mode: u8,
        num_bump_maps: u8,
        z_freeze: u8,
    ) {
        let mut gen_mode = bitfrob::u32_with_value(0, 3, 0u32, num_tex_coords.into());
        gen_mode = bitfrob::u32_with_value(4, 8, gen_mode, num_colors.into());
        gen_mode = bitfrob::u32_with_bit(9, gen_mode, ms_en);
        gen_mode = bitfrob::u32_with_value(10, 13, gen_mode, num_tex_stages.into());
        gen_mode = bitfrob::u32_with_value(14, 15, gen_mode, cull_mode.into());
        gen_mode = bitfrob::u32_with_value(16, 18, gen_mode, num_bump_maps.into());
        gen_mode = bitfrob::u32_with_value(19, 23, gen_mode, z_freeze.into());

        unsafe {
            let pipe_u8 = 0xCC00_8000 as *mut u8;
            let pipe_u32 = 0xCC00_8000 as *mut u32;

            pipe_u8.write_volatile(0x61);
            pipe_u32.write_volatile(gen_mode);
        }
    }

    pub fn copy_display(&mut self, framebuffer: &ViFramebuffer) {
        self.set_copy_display_source(0, 0, framebuffer.width, framebuffer.height);
        self.set_copy_display_distance(framebuffer.width);
        self.set_copy_display_destination(framebuffer);
    }
}

impl Default for Fifo {
    fn default() -> Self {
        Self::new()
    }
}

fn sync() {
    unsafe { core::arch::asm!("sc") }
}
