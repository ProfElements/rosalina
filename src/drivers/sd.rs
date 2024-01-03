use core::ptr::from_exposed_addr_mut;

use alloc::{boxed::Box, ffi::CString};
use bytemuck::Pod;

use crate::{
    cache::dc_flush_range,
    ios::IoVec,
    ipc::{
        rev2::IpcAccessMode,
        rev2::{IpcError, IpcRequest},
    },
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
    sd0_fd: u32,
    rca: Option<u32>,
    pub is_sdhc: bool,
}

impl SDCard {
    /// # Errors
    /// See `ipc::IpcError`
    pub fn new() -> Result<Self, IpcError> {
        let req = IpcRequest::open(
            CString::new("/dev/sdio/slot0").expect("sd card file name couldn't be constructed"),
            IpcAccessMode::ReadWrite,
        )
        .send()?;

        //Give back CString memory
        unsafe {
            let _ = CString::from_raw(from_exposed_addr_mut(
                (req.args[0] | 0x8000_0000)
                    .try_into()
                    .expect("Physical to virtaul address shift didn't work"),
            ));
        }

        let fd = req.ret;
        let mut sd_card = Self {
            sd0_fd: fd.try_into().unwrap(),
            is_sdhc: false,
            rca: None,
        };

        let _status = sd_card.reset()?;
        let _status = sd_card.status()?;

        sd_card.set_bus_width(4)?;
        sd_card.set_clock(true)?;

        sd_card.with_chip_select(|s| {
            s.set_block_length(512);
            s.set_bus_width_inner(4);
        });

        Ok(sd_card)
    }

    /// # Errors
    /// See `ipc::IpcError`
    fn reset(&mut self) -> Result<u32, IpcError> {
        const RESET_CARD: u32 = 4;

        if let Some(status) =
            IpcRequest::ioctl(self.sd0_fd, RESET_CARD, Box::new(()), Box::new(0u32))
                .send()?
                .take_output()
        {
            self.rca = Some(*status >> 16);
            Ok(*status)
        } else {
            Err(IpcError::Other(
                "Status was not able to be taken from the ipc request",
            ))
        }
    }

    /// # Errors
    /// See `ipc::IpcError`
    fn status(&mut self) -> Result<u32, IpcError> {
        const GET_STATUS: u32 = 0x0B;

        if let Some(status) =
            IpcRequest::ioctl(self.sd0_fd, GET_STATUS, Box::new(()), Box::new(0u32))
                .send()?
                .take_output::<u32>()
        {
            if *status & 0x100_000 == 0 {
                self.is_sdhc = true;
            }

            Ok(*status)
        } else {
            Err(IpcError::Other(
                "Status was not able to take from the ipc request",
            ))
        }
    }
    /// # Errors
    /// See `ipc::IpcError`
    fn get_host_controller_register(&mut self, reg: u32, size: u32) -> Result<u32, IpcError> {
        const READ_HC_REG: u32 = 2;

        IpcRequest::ioctl(
            self.sd0_fd,
            READ_HC_REG,
            Box::new([reg, 0, 0, size, 0, 0]),
            Box::new(0u32),
        )
        .send()?
        .take_output::<u32>()
        .map_or_else(
            || {
                Err(IpcError::Other(
                    "HCR was not able to be able to take from the ipc request",
                ))
            },
            |hcr| Ok(*hcr),
        )
    }

    /// # Errors
    /// See `ipc::IpcError`
    fn write_host_controller_register(
        &mut self,
        reg: u32,
        size: u32,
        data: u32,
    ) -> Result<(), IpcError> {
        const WRITE_HC_REG: u32 = 1;

        IpcRequest::ioctl(
            self.sd0_fd,
            WRITE_HC_REG,
            Box::new([reg, 0, 0, size, data, 0]),
            Box::new(()),
        )
        .send()
        .map(|_| ())
    }

    /// # Errors
    /// See `ipc::IpcError`
    fn get_host_control(&mut self) -> Result<u32, IpcError> {
        self.get_host_controller_register(0x28, 1)
    }

    /// # Errors
    /// See `ipc::IpcError`
    fn write_host_control(&mut self, size: u32, data: u32) -> Result<(), IpcError> {
        self.write_host_controller_register(0x28, size, data)
    }

    /// # Errors
    /// See `ipc::IpcError`
    fn set_bus_width(&mut self, bus_width: u32) -> Result<(), IpcError> {
        let mut hcr = self.get_host_control()?;

        hcr &= 0xff;
        hcr &= !0x2;

        if bus_width == 4 {
            hcr |= 0x2;
        }

        self.write_host_control(1, hcr)
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

    /// # Errors
    /// See `ipc::IpcError`
    fn set_clock(&mut self, is_set: bool) -> Result<(), IpcError> {
        const SET_CLK: u32 = 6;
        let clock = if is_set {
            Align32::new(1u32)
        } else {
            Align32::new(0u32)
        };

        IpcRequest::ioctl(self.sd0_fd, SET_CLK, Box::new(clock.inner), Box::new(()))
            .send()
            .map(|_| ())
    }

    fn select(&mut self) {
        self.card_cmd(SDIOCommand::ToggleSelect, self.rca.unwrap() << 16)
            .expect("Unable to select sd card");
    }

    fn deselect(&mut self) {
        self.card_cmd(SDIOCommand::ToggleSelect, 0)
            .expect("Unable to deselect sd card");
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
    /// See `ipc::IpcError`
    pub fn card_cmd(&mut self, cmd: SDIOCommand, arg: u32) -> Result<[u32; 4], IpcError> {
        const IOCTL_SEND_SDIO_CMD: u32 = 0x7;
        let cmd_u32: u32 = cmd.as_u32();
        let cmd_type = cmd.command_type() as u32;
        let resp_type = cmd.response_type() as u32;

        IpcRequest::ioctl(
            self.sd0_fd,
            IOCTL_SEND_SDIO_CMD,
            Box::new([cmd_u32, cmd_type, resp_type, arg, 0, 0, 0, 0, 0]),
            Box::new([0u32; 4]),
        )
        .send()?
        .take_output::<[u32; 4]>()
        .map_or_else(
            || {
                Err(IpcError::Other(
                    "response was not able to be take from ipc request",
                ))
            },
            |response| Ok(*response),
        )
    }

    /// # Errors
    /// See `ipc::IpcError`
    pub fn read_sectors(
        &mut self,
        sector_offset: usize,
        sectors: &mut [[u8; 512]],
    ) -> Result<[u32; 4], IpcError> {
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

            if let Ok(req) = IpcRequest::ioctlv::<2, 1>(
                s.sd0_fd,
                IOCTL_SEND_SDIO_CMD,
                Box::new([
                    IoVec::new(&mut sdio_cmd),
                    IoVec::new(&mut buffer),
                    IoVec::new(&mut sdio_resp),
                ]),
            )
            .send()
            {
                let _iovecs = unsafe {
                    Box::from_raw(from_exposed_addr_mut::<[IoVec; 3]>(
                        (req.args[3] | 0x8000_0000)
                            .try_into()
                            .expect("Unable to shift from phyiscal to virtual"),
                    ))
                };
            }
        });

        dc_flush_range(sectors.as_ptr().cast::<u8>(), sectors.len() * 512);

        Ok(sdio_resp)
    }

    /// # Errors
    /// See `ipc::IpcError`
    pub fn num_bytes(&mut self) -> Result<[u32; 4], IpcError> {
        const IOCTL_SEND_SDIO_CMD: u32 = 0x7;

        let sdio_command = SDIOCommand::SendCsd;
        let sdio_command_u32 = sdio_command.as_u32();
        let command_type = sdio_command.command_type() as u32;
        let response_type = sdio_command.response_type() as u32;
        let mut sdio_response: Result<[u32; 4], IpcError> = Ok([0u32; 4]);

        self.with_chip_select(|s| {
            if let Some(response) = IpcRequest::ioctl(
                s.sd0_fd,
                IOCTL_SEND_SDIO_CMD,
                Box::new([
                    sdio_command_u32,
                    command_type,
                    response_type,
                    s.rca.unwrap() << 16,
                    0,
                    0,
                    0,
                ]),
                Box::new([0u32; 4]),
            )
            .send()
            .expect("Unable to send ioctl")
            .take_output::<[u32; 4]>()
            {
                sdio_response = Ok(*response);
            } else {
                sdio_response = Err(IpcError::Other(
                    "Unable to take sdio response from ipc request",
                ));
            }
        });

        sdio_response
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
