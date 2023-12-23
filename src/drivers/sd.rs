use bytemuck::Pod;

use crate::{
    cache::dc_flush_range,
    ios::{self, FileAccessMode, IoVec, Ios},
};

#[repr(C, align(32))]
#[derive(Debug)]
pub struct Align32<T: Pod> {
    inner: T,
}

impl<T: Pod> Align32<T> {
    pub const fn new(inner: T) -> Self {
        Self { inner }
    }
}

pub struct SDCard {
    sd0_fd: Ios,
    rca: Option<u32>,
    pub is_sdhc: bool,
}

type Error = ios::Error;

impl SDCard {
    /// # Errors
    /// See `ios::Error`
    pub fn new() -> Result<Self, Error> {
        let sd0_fd = Ios::open("/dev/sdio/slot0", FileAccessMode::Read)?;

        let mut sd_card = Self {
            sd0_fd,
            is_sdhc: false,
            rca: None,
        };

        let _status = sd_card.reset();
        let _status = sd_card.status();
        sd_card.set_bus_width(4);
        sd_card.set_clock(true);
        sd_card.with_chip_select(|s| {
            s.set_block_length(512);
            s.set_bus_width_inner(4);
        });

        Ok(sd_card)
    }

    fn reset(&mut self) -> u32 {
        const RESET_CARD: u32 = 4;
        let mut status = Align32::new(0u32);
        self.sd0_fd
            .ioctl(RESET_CARD, &(), &mut status)
            .expect("SDIO could not be reset");

        self.rca = Some(status.inner >> 16);

        status.inner
    }

    fn status(&mut self) -> u32 {
        const GET_STATUS: u32 = 0x0B;
        let mut status = Align32::new(0u32);
        self.sd0_fd
            .ioctl(GET_STATUS, &(), &mut status)
            .expect("SDIO could not grab card status");

        let stat = status.inner;

        if stat & 0x100_000 > 0 {
            self.is_sdhc = true;
        }

        stat
    }

    fn get_host_controller_register(&mut self, reg: u32, size: u32) -> u32 {
        const READ_HC_REG: u32 = 2;
        let mut hcr_value = Align32::new(0u32);
        let hcr_query = Align32::new([reg, 0, 0, size, 0, 0]);
        self.sd0_fd
            .ioctl(READ_HC_REG, &hcr_query, &mut hcr_value)
            .expect("SDIO could not grab host controller register value");

        hcr_value.inner
    }

    fn write_host_controller_register(&mut self, reg: u32, size: u32, data: u32) {
        const WRITE_HC_REG: u32 = 1;
        let hcr_query = Align32::new([reg, 0, 0, size, data, 0]);
        self.sd0_fd
            .ioctl(WRITE_HC_REG, &hcr_query, &mut ())
            .expect("SDIO could not grab host controller register value");
    }

    fn get_host_control(&mut self) -> u32 {
        self.get_host_controller_register(0x28, 1)
    }

    fn write_host_control(&mut self, size: u32, data: u32) {
        self.write_host_controller_register(0x28, size, data);
    }

    fn set_bus_width(&mut self, bus_width: u32) {
        let mut hcr = self.get_host_control();

        hcr &= 0xff;
        hcr &= !0x2;

        if bus_width == 4 {
            hcr |= 0x2;
        }

        self.write_host_control(1, hcr);
    }

    fn set_bus_width_inner(&mut self, bus_width: u32) {
        self.card_cmd(SDIOCommand::AppCmd, self.rca.unwrap() << 16)
            .unwrap();

        self.card_cmd(
            SDIOCommand::AppCommand(SDIOAppCommand::SetBusWidth),
            if bus_width == 4 { 0x2 } else { 0x0 },
        )
        .unwrap();
    }

    fn set_clock(&mut self, is_set: bool) {
        const SET_CLK: u32 = 6;
        let clock = if is_set {
            Align32::new(1u32)
        } else {
            Align32::new(0u32)
        };

        self.sd0_fd
            .ioctl(SET_CLK, &clock, &mut ())
            .expect("SDIO could not set clock");
    }

    fn select(&mut self) {
        self.card_cmd(SDIOCommand::ToggleSelect, self.rca.unwrap() << 16)
            .unwrap();
    }

    fn deselect(&mut self) {
        self.card_cmd(SDIOCommand::ToggleSelect, 0).unwrap();
    }

    pub fn with_chip_select<F>(&mut self, func: F)
    where
        F: FnOnce(&mut Self),
    {
        self.select();
        func(self);
        self.deselect();
    }

    fn set_block_length(&mut self, blk_len: u32) {
        self.card_cmd(SDIOCommand::SetBlockLength, blk_len).unwrap();
    }

    /// # Errors
    /// See `ios::Error`
    pub fn card_cmd(&mut self, cmd: SDIOCommand, arg: u32) -> Result<[u32; 4], Error> {
        const IOCTL_SEND_SDIO_CMD: u32 = 0x7;
        let cmd_u32: u32 = cmd.as_u32();
        let cmd_type = cmd.command_type() as u32;
        let resp_type = cmd.response_type() as u32;

        let sdio_cmd = Align32::new([cmd_u32, cmd_type, resp_type, arg, 0, 0, 0, 0, 0]);

        let mut sdio_resp = Align32::new([0u32; 4]);

        self.sd0_fd
            .ioctl(IOCTL_SEND_SDIO_CMD, &sdio_cmd, &mut sdio_resp)?;

        let inner = sdio_resp.inner;
        println!("{inner:?}");

        Ok(sdio_resp.inner)
    }

    pub fn read_sectors(
        &mut self,
        sector_offset: usize,
        sectors: &mut [[u8; 512]],
    ) -> Result<[u32; 4], Error> {
        const IOCTL_SEND_SDIO_CMD: u32 = 0x7;
        let sector_offset = if self.is_sdhc {
            sector_offset
        } else {
            sector_offset * 512
        };
        let cmd = SDIOCommand::ReadMultiBlock;
        let cmd_u32 = cmd.as_u32();
        let cmd_type = cmd.command_type() as u32;
        let resp_type = cmd.response_type() as u32;
        let mut sdio_resp = [0u32; 4];
        self.with_chip_select(|s| {
            let mut sdio_cmd = [
                cmd_u32,
                cmd_type,
                resp_type,
                sector_offset.try_into().unwrap(),
                sectors.len().try_into().unwrap(),
                512,
                sectors.as_mut_ptr().addr().try_into().unwrap(),
                0,
                0,
            ];
            let mut buffer = [
                sectors.as_mut_ptr().addr().try_into().unwrap(),
                u32::try_from(sectors.len() * 512).unwrap(),
            ];

            s.sd0_fd
                .ioctlv(
                    IOCTL_SEND_SDIO_CMD,
                    2,
                    1,
                    &mut [
                        IoVec::new(&mut sdio_cmd),
                        IoVec::new(&mut buffer),
                        IoVec::new(&mut sdio_resp),
                    ],
                )
                .unwrap();
        });

        dc_flush_range(sectors.as_ptr().cast::<u8>(), sectors.len() * 512);

        Ok(sdio_resp)
    }

    pub fn num_bytes(&mut self) -> Result<[u32; 4], Error> {
        const IOCTL_SEND_SDIO_CMD: u32 = 0x7;

        let sdio_command = SDIOCommand::SendCsd;
        let sdio_command_u32 = sdio_command.as_u32();
        let command_type = sdio_command.command_type() as u32;
        let response_type = sdio_command.response_type() as u32;
        let mut sdio_response = [0u32; 4];
        self.with_chip_select(|s| {
            let mut sdio_cmd = [
                sdio_command_u32,
                command_type,
                response_type,
                s.rca.unwrap() << 16,
                0,
                0,
                0,
            ];

            s.sd0_fd
                .ioctl(IOCTL_SEND_SDIO_CMD, &mut sdio_cmd, &mut sdio_response)
                .expect("SDIO FAILED");
        });

        Ok(sdio_response)
    }
}

#[repr(u32)]
pub enum SDIOCommandType {
    Bc = 1,
    Bcr = 2,
    Ac = 3,
    Adtc = 4,
    Unknown = 0xFF,
}

#[repr(u32)]
pub enum SDIOResponseType {
    None = 0,
    R1 = 1,
    R1B = 2,
    R2 = 3,
    R3 = 4,
    R4 = 5,
    R5 = 6,
    R6 = 7,
    R7,
    Unknown = 0xFF,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum SDIOCommand {
    GoIdle = 0x0,
    SendOpCommand = 0x1,
    AllSendCid = 0x2, // NOT SUPPORTED BY SDIO
    SendRelativeAddr = 0x3,
    SetDsr = 0x4, //NOT SUPPORTED BY SDIO
    IoSendOpCond = 0x5,
    SwitchFunc = 0x6,
    ToggleSelect = 0x7,
    SendIfCommand = 0x8,
    SendCsd = 0x9, // NOT SUPPORTED BY SDIO,
    SendCid = 10,  // NOT SUPPORTED BY SDIO,
    VoltageSwitch = 11,
    StopTransmission = 12,
    SendStatus = 13,
    GoInactiveState = 15,
    SetBlockLength = 16,
    ReadSingleBlock = 17,
    ReadMultiBlock = 18,
    SendTuningBlock = 19,
    SpeedClassControl = 20,
    SetBlockCount = 23,
    WriteSingleBlock = 24,
    WriteMultiBlock = 25,
    ProgramCsd = 27, //NOT SUPPORTED BY SDIO
    SetWriteProt = 28,
    ClrWriteProt = 29,
    SendWriteProt = 30,
    EraseWriteBlockStart = 32,
    EraseWriteBlockEnd = 33,
    Erase = 38,
    ToggleLock = 42,
    ReadWriteDirect = 52,
    ReadWriteExtended = 53,
    AppCmd = 55,
    GenCmd = 56,
    ReadOcr = 58,
    CRCOnOff = 59,
    AppCommand(SDIOAppCommand),
}

impl SDIOCommand {
    pub const fn as_u32(self) -> u32 {
        match self {
            Self::AppCommand(c) => c.as_u32(),
            Self::GoIdle => 0x0,
            Self::SendOpCommand => 0x1,
            Self::AllSendCid => 0x2, // NOT SUPPORTED BY SDIO
            Self::SendRelativeAddr => 0x3,
            Self::SetDsr => 0x4, //NOT SUPPORTED BY SDIO
            Self::IoSendOpCond => 0x5,
            Self::SwitchFunc => 0x6,
            Self::ToggleSelect => 0x7,
            Self::SendIfCommand => 0x8,
            Self::SendCsd => 0x9, // NOT SUPPORTED BY SDIO,
            Self::SendCid => 10,  // NOT SUPPORTED BY SDIO,
            Self::VoltageSwitch => 11,
            Self::StopTransmission => 12,
            Self::SendStatus => 13,
            Self::GoInactiveState => 15,
            Self::SetBlockLength => 16,
            Self::ReadSingleBlock => 17,
            Self::ReadMultiBlock => 18,
            Self::SendTuningBlock => 19,
            Self::SpeedClassControl => 20,
            Self::SetBlockCount => 23,
            Self::WriteSingleBlock => 24,
            Self::WriteMultiBlock => 25,
            Self::ProgramCsd => 27, //NOT SUPPORTED BY SDIO
            Self::SetWriteProt => 28,
            Self::ClrWriteProt => 29,
            Self::SendWriteProt => 30,
            Self::EraseWriteBlockStart => 32,
            Self::EraseWriteBlockEnd => 33,
            Self::Erase => 38,
            Self::ToggleLock => 42,
            Self::ReadWriteDirect => 52,
            Self::ReadWriteExtended => 53,
            Self::AppCmd => 55,
            Self::GenCmd => 56,
            Self::ReadOcr => 58,
            Self::CRCOnOff => 59,
        }
    }

    pub const fn command_type(self) -> SDIOCommandType {
        match self {
            Self::GoIdle | Self::SetDsr => SDIOCommandType::Bc,
            Self::AllSendCid | Self::SendRelativeAddr | Self::SendIfCommand => SDIOCommandType::Bcr,
            Self::SendCsd
            | Self::ToggleSelect
            | Self::SendCid
            | Self::StopTransmission
            | Self::SendStatus
            | Self::SetBlockLength
            | Self::SpeedClassControl
            | Self::SetBlockCount
            | Self::SetWriteProt
            | Self::ClrWriteProt
            | Self::Erase
            | Self::EraseWriteBlockStart
            | Self::EraseWriteBlockEnd
            | Self::AppCmd
            | Self::GoInactiveState
            | Self::VoltageSwitch => SDIOCommandType::Ac,
            Self::ReadSingleBlock
            | Self::ReadMultiBlock
            | Self::SendTuningBlock
            | Self::WriteSingleBlock
            | Self::WriteMultiBlock
            | Self::ProgramCsd
            | Self::SendWriteProt
            | Self::ToggleLock
            | Self::GenCmd
            | Self::SwitchFunc
            | Self::ReadWriteDirect
            | Self::ReadWriteExtended
            | Self::ReadOcr
            | Self::CRCOnOff => SDIOCommandType::Adtc,
            Self::AppCommand(c) => c.command_type(),
            _ => SDIOCommandType::Unknown,
        }
    }

    pub const fn response_type(self) -> SDIOResponseType {
        match self {
            Self::GoIdle | Self::SetDsr | Self::GoInactiveState => SDIOResponseType::None,
            Self::AllSendCid | Self::SendStatus => SDIOResponseType::R2,
            Self::SendRelativeAddr => SDIOResponseType::R6,
            Self::ToggleSelect
            | Self::StopTransmission
            | Self::SpeedClassControl
            | Self::SetWriteProt
            | Self::ClrWriteProt
            | Self::SendWriteProt => SDIOResponseType::R1B,
            Self::SendIfCommand => SDIOResponseType::R7,
            Self::ProgramCsd
            | Self::WriteMultiBlock
            | Self::WriteSingleBlock
            | Self::ReadMultiBlock
            | Self::ReadSingleBlock
            | Self::SetBlockLength
            | Self::GenCmd
            | Self::AppCmd
            | Self::Erase
            | Self::EraseWriteBlockEnd
            | Self::EraseWriteBlockStart
            | Self::SwitchFunc
            | Self::VoltageSwitch
            | Self::CRCOnOff
            | Self::SendCsd
            | Self::SendCid
            | Self::SetBlockCount
            | Self::ToggleLock
            | Self::SendTuningBlock => SDIOResponseType::R1,
            Self::ReadOcr => SDIOResponseType::R3,
            Self::ReadWriteDirect | Self::ReadWriteExtended => SDIOResponseType::R5,
            Self::AppCommand(c) => c.response_type(),
            _ => SDIOResponseType::Unknown,
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum SDIOAppCommand {
    SetBusWidth = 6,
    SdStatus = 13,
    SendNumWriteBlocks = 22,
    SetWriteBlockEraseCount = 23,
    SdAppOpCommand = 31,
    SetClearCardDetect = 42,
    SendSCR = 51, // NOT SUPPORTED BY SDIO
}

impl SDIOAppCommand {
    pub const fn as_u32(self) -> u32 {
        match self {
            Self::SetBusWidth => 6,
            Self::SdStatus => 13,
            Self::SendNumWriteBlocks => 22,
            Self::SetWriteBlockEraseCount => 23,
            Self::SdAppOpCommand => 31,
            Self::SetClearCardDetect => 42,
            Self::SendSCR => 41,
        }
    }

    pub const fn command_type(self) -> SDIOCommandType {
        match self {
            Self::SetClearCardDetect | Self::SetBusWidth | Self::SetWriteBlockEraseCount => {
                SDIOCommandType::Ac
            }
            Self::SdStatus | Self::SendNumWriteBlocks | Self::SendSCR => SDIOCommandType::Adtc,
            Self::SdAppOpCommand => SDIOCommandType::Bcr,
        }
    }

    pub const fn response_type(self) -> SDIOResponseType {
        match self {
            Self::SetBusWidth
            | Self::SdStatus
            | Self::SendNumWriteBlocks
            | Self::SetWriteBlockEraseCount
            | Self::SetClearCardDetect
            | Self::SendSCR => SDIOResponseType::R1,
            Self::SdAppOpCommand => SDIOResponseType::R3,
        }
    }
}
