use crate::{
    cache::dc_invalidate_range,
    mmio::{
        exi::{
            DmaMode, DmaStart, ExiClock, ExiControl, ExiDevice, ExiParams, ReadWriteMode,
            EXI_CHANNEL_0_DMA_LENGTH, EXI_CHANNEL_0_DMA_START, EXI_CHANNEL_0_IMM_DATA,
        },
        pi::Mask,
    },
};

pub struct Sram {
    pub(crate) buffer: [u8; 64],
}

impl Sram {
    pub fn init() -> Self {
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

        Self { buffer }
    }
}
