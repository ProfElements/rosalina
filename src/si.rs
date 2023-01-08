use crate::{
    interrupts::Interrupt,
    mmio::{
        exi::DmaStart,
        pi::{InterruptMask, InterruptState, Mask},
        si::{
            ErrorStatus, ExiLock, SiChannel, SiComm, SiInputBufHi, SiInputBufLo, SiOutputBuf,
            SiPoll, SiStatus, SI_BUF,
        },
        vi::Enabled,
    },
    DOLPHIN_HLE,
};

use core::fmt::Write;

pub struct SerialInterface;

impl SerialInterface {
    pub fn init() -> Self {
        SiPoll::new()
            .with_x_poll_time(246)
            .with_y_poll_time(2)
            .write();

        while SiComm::read().command_state().into() {}

        SiComm::new()
            .with_transfer_complete_interrupt_status(InterruptState::Happened)
            .write();

        ExiLock::new()
            .with_exi_32hertz_enabled(Enabled::Enabled)
            .write();

        Interrupt::set_interrupt_handler(Interrupt::SerialInterface, |_| {
            let mut comm = SiComm::read();

            if comm.read_status_interrupt_status().into()
                && comm.read_status_interrupt_mask().into()
            {
                comm.with_read_status_interrupt_status(InterruptState::Happened)
                    .write();
            }

            if comm.transfer_complete_interrupt_status().into()
                && comm.transfer_complete_interrupt_mask().into()
            {
                comm.with_transfer_complete_interrupt_status(InterruptState::Happened)
                    .write();
            }

            Ok(())
        });

        InterruptMask::read()
            .with_serial_interface(Mask::Enabled)
            .write();

        Self
    }

    /// # Errors
    ///
    /// This can produce 4 errors.
    /// `NoResponse`: there is no available serial device on that channel
    /// `Overrun`: you had a buffer overrun
    /// `Underrun`: you had a buffer underrun
    /// `Collision`: your values are getting modified while read hopefully this doesnt happen
    pub fn get_type(si_channel: SiChannel) -> Result<u32, TransferError> {
        match si_channel {
            SiChannel::Zero => SiOutputBuf::new()
                .with_cmd(0x0)
                .with_output_zero(0x0)
                .with_output_one(0x0)
                .write_zero(),
            SiChannel::One => SiOutputBuf::new()
                .with_cmd(0x0)
                .with_output_zero(0x0)
                .with_output_one(0x0)
                .write_one(),
            SiChannel::Two => SiOutputBuf::new()
                .with_cmd(0x0)
                .with_output_zero(0x0)
                .with_output_one(0x0)
                .write_two(),
            SiChannel::Three => SiOutputBuf::new()
                .with_cmd(0x0)
                .with_output_zero(0x0)
                .with_output_one(0x0)
                .write_three(),
        }

        unsafe {
            core::arch::asm!("sync");
            core::arch::asm!("isync");
        }

        SiComm::new()
            .with_si_channel(SiChannel::Zero)
            .with_input_length(1)
            .with_output_length(1)
            .with_command_enabled(Enabled::Enabled)
            .with_channel_enabled(Enabled::Enabled)
            .with_transfer_complete_interrupt_mask(Mask::Enabled)
            .with_dma_start(DmaStart::Start)
            .write();

        while SiComm::read().command_state().into() {}
        SiComm::read()
            .with_transfer_complete_interrupt_status(InterruptState::Happened)
            .write();

        let input_buf = match si_channel {
            SiChannel::Zero => SiInputBufHi::read_zero(),
            SiChannel::One => SiInputBufHi::read_one(),
            SiChannel::Two => SiInputBufHi::read_two(),
            SiChannel::Three => SiInputBufHi::read_three(),
        };

        let _lo = match si_channel {
            SiChannel::Zero => SiInputBufLo::read_zero(),
            SiChannel::One => SiInputBufLo::read_one(),
            SiChannel::Two => SiInputBufLo::read_two(),
            SiChannel::Three => SiInputBufLo::read_three(),
        };

        if !bool::from(input_buf.error_latch()) && !bool::from(input_buf.error_status()) {
            return Ok(SI_BUF.get(0).unwrap().read());
        }

        unsafe {
            write!(DOLPHIN_HLE, "GOT ERROR").unwrap();
        }

        let mut status = SiStatus::read();
        let res = match si_channel {
            SiChannel::Zero => {
                if status.channel_0_no_response().into() {
                    status
                        .with_chan_0_no_response(ErrorStatus::Happened)
                        .write();
                    return Err(TransferError::NoResponse);
                }
                if status.channel_0_overrun().into() {
                    status.with_chan_0_overrun(ErrorStatus::Happened).write();
                    return Err(TransferError::Overrun);
                }
                if status.channel_0_underrun().into() {
                    status.with_chan_0_underrun(ErrorStatus::Happened).write();
                    return Err(TransferError::Underrun);
                }
                if status.channel_0_collision().into() {
                    status.with_chan_0_collision(ErrorStatus::Happened).write();
                    return Err(TransferError::Collision);
                }
                Ok(0)
            }
            SiChannel::One => {
                if status.channel_1_no_response().into() {
                    status
                        .with_chan_1_no_response(ErrorStatus::Happened)
                        .write();
                    return Err(TransferError::NoResponse);
                }
                if status.channel_1_overrun().into() {
                    status.with_chan_1_overrun(ErrorStatus::Happened).write();
                    return Err(TransferError::Overrun);
                }
                if status.channel_1_underrun().into() {
                    status.with_chan_1_underrun(ErrorStatus::Happened).write();
                    return Err(TransferError::Underrun);
                }
                if status.channel_1_collision().into() {
                    status.with_chan_1_collision(ErrorStatus::Happened).write();
                    return Err(TransferError::Collision);
                }
                Ok(0)
            }
            SiChannel::Two => {
                if status.channel_2_no_response().into() {
                    status
                        .with_chan_2_no_response(ErrorStatus::Happened)
                        .write();
                    return Err(TransferError::NoResponse);
                }
                if status.channel_2_overrun().into() {
                    status.with_chan_2_overrun(ErrorStatus::Happened).write();
                    return Err(TransferError::Overrun);
                }
                if status.channel_2_underrun().into() {
                    status.with_chan_2_underrun(ErrorStatus::Happened).write();
                    return Err(TransferError::Underrun);
                }
                if status.channel_2_collision().into() {
                    status.with_chan_2_collision(ErrorStatus::Happened).write();
                    return Err(TransferError::Collision);
                }
                Ok(0)
            }
            SiChannel::Three => {
                if status.channel_3_no_response().into() {
                    status
                        .with_chan_3_no_response(ErrorStatus::Happened)
                        .write();
                    return Err(TransferError::NoResponse);
                }
                if status.channel_3_overrun().into() {
                    status.with_chan_3_overrun(ErrorStatus::Happened).write();
                    return Err(TransferError::Overrun);
                }
                if status.channel_3_underrun().into() {
                    status.with_chan_3_underrun(ErrorStatus::Happened).write();
                    return Err(TransferError::Underrun);
                }
                if status.channel_3_collision().into() {
                    status.with_chan_3_collision(ErrorStatus::Happened).write();
                    return Err(TransferError::Collision);
                }
                Ok(0)
            }
        };

        res
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TransferError {
    Underrun,
    Overrun,
    Collision,
    NoResponse,
}
