use crate::{
    cache::dc_invalidate_range,
    interrupts::Interrupt,
    mmio::{
        exi::{
            DmaMode, DmaStart, ExiClock, ExiControl, ExiDevice, ExiParams, ReadWriteMode,
            RomDiscramble, EXI_CHANNEL_0_DMA_LENGTH, EXI_CHANNEL_0_DMA_START,
            EXI_CHANNEL_0_IMM_DATA,
        },
        pi::{InterruptState, Mask},
    },
    sram::Sram,
};

pub struct ExternalInterface;

impl ExternalInterface {
    pub fn init() {
        ExiParams::new().write_zero();
        ExiParams::new().write_one();
        ExiParams::new().write_two();

        ExiParams::new()
            .with_rom_discramble(RomDiscramble::Disabled)
            .write_zero();

        Interrupt::set_interrupt_handler(Interrupt::ExternalInterface, |_| {
            let mut params = ExiParams::read_zero();
            if params.has_interrupts() {
                if params.exi_status() == InterruptState::Happened {
                    params
                        .with_exi_status(InterruptState::Happened)
                        .write_zero();
                }

                if params.transfer_complete_status() == InterruptState::Happened {
                    params
                        .with_transfer_complete_status(InterruptState::Happened)
                        .write_zero();
                }

                if params.external_insertion_status() == InterruptState::Happened {
                    params
                        .with_transfer_complete_status(InterruptState::Happened)
                        .write_zero();
                }
            }

            params = ExiParams::read_one();
            if params.has_interrupts() {
                if params.exi_status() == InterruptState::Happened {
                    params.with_exi_status(InterruptState::Happened).write_one();
                }

                if params.transfer_complete_status() == InterruptState::Happened {
                    params
                        .with_transfer_complete_status(InterruptState::Happened)
                        .write_one();
                }

                if params.external_insertion_status() == InterruptState::Happened {
                    params
                        .with_external_insertion_status(InterruptState::Happened)
                        .write_one();
                }
            }

            params = ExiParams::read_two();
            if params.has_interrupts() {
                if params.exi_status() == InterruptState::Happened {
                    params.with_exi_status(InterruptState::Happened).write_two();
                }

                if params.transfer_complete_status() == InterruptState::Happened {
                    params
                        .with_transfer_complete_status(InterruptState::Happened)
                        .write_two();
                }

                if params.external_insertion_status() == InterruptState::Happened {
                    params
                        .with_external_insertion_status(InterruptState::Happened)
                        .write_two();
                }
            }

            Ok(())
        })
    }

    pub fn get_rtc() -> u32 {
        ExiParams::read_zero()
            .with_device_select(ExiDevice::Device1)
            .with_clock(ExiClock::EightMegahertz)
            .with_exi_interrupt_mask(Mask::Enabled)
            .with_transfer_complete_mask(Mask::Enabled)
            .with_external_insertion_mask(Mask::Enabled)
            .write_zero();

        EXI_CHANNEL_0_IMM_DATA.write(0x20_00_00_00u32);

        ExiControl::read_zero()
            .with_dma_mode(DmaMode::Immediate)
            .with_read_write_mode(ReadWriteMode::Write)
            .with_transfer_length(4)
            .with_dma_start(DmaStart::Start)
            .write_zero();

        while ExiControl::read_zero().dma_start() == DmaStart::Start {}

        ExiControl::read_zero()
            .with_dma_mode(DmaMode::Immediate)
            .with_read_write_mode(ReadWriteMode::Read)
            .with_transfer_length(4)
            .with_dma_start(DmaStart::Start)
            .write_zero();

        while ExiControl::read_zero().dma_start() == DmaStart::Start {}

        ExiParams::read_zero()
            .with_device_select(ExiDevice::None)
            .write_zero();

        EXI_CHANNEL_0_IMM_DATA.read()
    }

    pub fn get_sram() -> Sram {
        let mut buffer: [u8; 64] = [0; 64];
        dc_invalidate_range(buffer.as_mut_ptr(), buffer.len());

        ExiParams::read_zero()
            .with_device_select(ExiDevice::Device1)
            .with_clock(ExiClock::EightMegahertz)
            .with_exi_interrupt_mask(Mask::Enabled)
            .with_transfer_complete_mask(Mask::Enabled)
            .with_external_insertion_mask(Mask::Enabled)
            .write_zero();

        EXI_CHANNEL_0_IMM_DATA.write(0x20000100);

        ExiControl::read_zero()
            .with_dma_mode(DmaMode::Immediate)
            .with_read_write_mode(ReadWriteMode::Write)
            .with_transfer_length(4)
            .with_dma_start(DmaStart::Start)
            .write_zero();

        while ExiControl::read_zero().dma_start() == DmaStart::Start {}

        EXI_CHANNEL_0_DMA_START.write(buffer.as_ptr().addr());
        EXI_CHANNEL_0_DMA_LENGTH.write(64);

        ExiControl::read_zero()
            .with_dma_mode(DmaMode::Dma)
            .with_read_write_mode(ReadWriteMode::Read)
            .with_dma_start(DmaStart::Start)
            .write_zero();

        while ExiControl::read_zero().dma_start() == DmaStart::Start {}

        ExiParams::read_zero()
            .with_device_select(ExiDevice::None)
            .write_zero();

        Sram { buffer }
    }
}
