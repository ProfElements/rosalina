use core::{alloc::Layout, pin::Pin};

use alloc::boxed::Box;
use bit_field::BitField;

use crate::{
    interrupts::Interrupt,
    mmio::{
        cp::{
            CommandClear, CommandControl, CommandStatus, CP_FIFO_BASE_HI, CP_FIFO_BASE_LO,
            CP_FIFO_END_HI, CP_FIFO_END_LO, CP_FIFO_HIGH_MARK_HI, CP_FIFO_HIGH_MARK_LO,
            CP_FIFO_LO_MARK_HI, CP_FIFO_LO_MARK_LO, CP_FIFO_READ_PTR_HI, CP_FIFO_READ_PTR_LO,
            CP_FIFO_READ_WRITE_DST_HI, CP_FIFO_READ_WRITE_DST_LO, CP_FIFO_WRITE_PTR_HI,
            CP_FIFO_WRITE_PTR_LO,
        },
        pi::{InterruptMask, InterruptState, Mask, FIFO_BASE, FIFO_END, FIFO_WRITE_PTR},
        vi::Enabled,
        Physical,
    },
};

pub fn init() {
    Interrupt::set_interrupt_handler(Interrupt::CommandProcessor, |_| {
        let status = CommandStatus::read();

        if status.fifo_underflow() {
            CommandControl::read()
                .with_fifo_underflow_interrupt(InterruptState::Happened)
                .write();
            CommandClear::read().with_fifo_underflow(true).write();
        }

        if status.fifo_overflow() {
            CommandControl::read()
                .with_fifo_overflow_interrupt(InterruptState::Happened)
                .write();
            CommandClear::read().with_fifo_overflow(true).write();
        }

        if status.fifo_breakpoint_interrupt().into() {
            CommandControl::read()
                .with_fifo_breakpoint_interrupt(InterruptState::Happened)
                .write();
        }

        Ok(())
    });

    InterruptMask::read()
        .with_command_fifo(Mask::Enabled)
        .write();
}

#[repr(C, align(32))]
pub struct Fifo<const SIZE: usize> {
    buf: Pin<Box<[u8; SIZE]>>,
}

impl<const SIZE: usize> Fifo<SIZE> {
    pub fn new() -> Self {
        let buf = Pin::new(unsafe {
            Box::from_raw(
                core::ptr::slice_from_raw_parts_mut(
                    alloc::alloc::alloc(Layout::from_size_align(SIZE, 32).unwrap()),
                    SIZE,
                )
                .cast::<[u8; SIZE]>(),
            )
        });
        Self { buf }
    }

    pub fn link_pi(&mut self) {
        let buf_start_addr = Physical::new(self.buf.as_mut_ptr()).addr();
        let buf_end_addr =
            Physical::new(unsafe { self.buf.as_mut_ptr().add(self.buf.len()) }).addr();
        let buf_write_addr = buf_start_addr;

        FIFO_BASE.write(buf_start_addr);
        FIFO_END.write(buf_end_addr);
        FIFO_WRITE_PTR.write(buf_write_addr);
    }

    pub fn link_cp(&mut self) {
        let buf_start_addr = Physical::new(self.buf.as_mut_ptr()).addr();
        let buf_end_addr =
            Physical::new(unsafe { self.buf.as_mut_ptr().add(self.buf.len()) }).addr();
        let buf_write_addr = buf_start_addr;
        let buf_read_addr = buf_start_addr;

        let read_write_dst = buf_write_addr - buf_read_addr;

        let high = self.buf.len() - 16 * 1024;
        let lo = self.buf.len() >> 1;

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

        unsafe { core::arch::asm!("sc") };
    }

    pub fn confirm_link(&self) {
        CommandClear::new()
            .with_fifo_overflow(true)
            .with_fifo_underflow(true)
            .write();
        CommandControl::read()
            .with_fifo_link_enable(Enabled::Enabled)
            .with_fifo_overflow_interrupt(InterruptState::Happened)
            .with_gp_fifo_read_enable(Enabled::Enabled)
            .write();
    }
}

impl<const SIZE: usize> Default for Fifo<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn enable_write_gather_pipe() {
    let val: usize = 0x0C00_8000;
    unsafe {
        core::arch::asm!(
            "mtspr 921,{val}",
            "mfspr {val}, 920",
            "mtspr 920,{val2}",
            val = in(reg) val,
            val2 = in(reg) val | 0x4000_0000,
        );
    }
}
