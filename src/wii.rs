use alloc::{ffi::CString, vec::Vec};

use crate::ipc::rev2::{IpcAccessMode, IpcRequest};

#[repr(C)]
pub struct StateFlags {
    checksum: usize,
    flags: u8,
    kind: u8,
    disc_state: u8,
    return_to: u8,
    unkown: [u32; 6],
}

#[derive(Debug)]
pub struct InvalidStateFlagsError;

impl TryFrom<&[u8; 32]> for StateFlags {
    type Error = InvalidStateFlagsError;
    fn try_from(value: &[u8; 32]) -> Result<Self, Self::Error> {
        let mut unkown_buf = [0u32; 6];

        for (n, unk) in value[7..]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|val| u32::from_be_bytes(*val))
            .enumerate()
        {
            unkown_buf[n] = unk;
        }

        Ok(Self {
            checksum: usize::from_be_bytes(value[0..=3].try_into().unwrap()),
            flags: value[4],
            kind: value[5],
            disc_state: value[6],
            return_to: value[7],
            unkown: unkown_buf,
        })
    }
}

#[repr(C)]
pub struct NandBootInfo {
    checksum: usize,
    args_offset: u32,
    unk: [u8; 2],
    app_type: u8,
    title_type: u8,
    launch_code: u32,
    unknown: [u32; 2],
    args_buffer: [u8; 0x1000],
}

#[derive(Debug)]
pub struct InvalidNandBootInfoError;
impl TryFrom<&[u8; 4120]> for NandBootInfo {
    type Error = InvalidNandBootInfoError;
    fn try_from(value: &[u8; 4120]) -> Result<Self, Self::Error> {
        Ok(Self {
            checksum: usize::from_be_bytes(value[0..=3].try_into().unwrap()),
            args_offset: u32::from_be_bytes(value[4..=7].try_into().unwrap()),
            unk: [value[8], value[9]],
            app_type: value[10],
            title_type: value[11],
            launch_code: u32::from_be_bytes(value[12..=15].try_into().unwrap()),
            unknown: [
                u32::from_be_bytes(value[16..=20].try_into().unwrap()),
                u32::from_be_bytes(value[21..=24].try_into().unwrap()),
            ],
            args_buffer: value[24..].try_into().unwrap(),
        })
    }
}

pub struct Wii {
    state: Option<StateFlags>,
    nand_boot_info: Option<NandBootInfo>,
}

impl Wii {
    pub fn init() -> Self {
        let state_buf = Vec::with_capacity(core::mem::size_of::<StateFlags>());
        let nand_info_buf = Vec::with_capacity(4120);

        let mut flags = None;
        let mut info = None;

        if let Ok(req) = IpcRequest::open(
            CString::new("/title/00000001/00000002/data/state.dat").unwrap(),
            IpcAccessMode::Read,
        )
        .send()
        {
            let fd = req.ret;
            if let Ok(mut req) = IpcRequest::read(fd.try_into().unwrap(), state_buf).send() {
                if let Some(buf) = req.take_buf() {
                    let state_data_buf: &[u8; 32] = buf.as_slice().try_into().unwrap();
                    flags = Some(StateFlags::try_from(state_data_buf).unwrap());
                    if !Self::valid_checksum(state_data_buf) {
                        crate::println!("Invalid checksum for state data buf");
                    }
                };
            }
        }

        if let Ok(req) = IpcRequest::open(
            CString::new("/shared2/sys/NANDBOOTINFO").unwrap(),
            IpcAccessMode::Read,
        )
        .send()
        {
            let fd = req.ret;
            if let Ok(mut req) = IpcRequest::read(fd.try_into().unwrap(), nand_info_buf).send() {
                if let Some(buf) = req.take_buf() {
                    let info_buf: &[u8; 4120] = buf.as_slice().try_into().unwrap();
                    info = Some(NandBootInfo::try_from(info_buf).unwrap());
                    if !Self::valid_checksum(info_buf) {
                        crate::println!("Invalid checksum for info buf");
                    }
                }
            }
        }

        Self {
            state: flags,
            nand_boot_info: info,
        }
    }

    pub const fn state(&self) -> &Option<StateFlags> {
        &self.state
    }

    pub const fn info(&self) -> &Option<NandBootInfo> {
        &self.nand_boot_info
    }

    fn valid_checksum(buf: &[u8]) -> bool {
        //THIS ASSUME THE CHECKSUM IS THE FIRST 4 bytes of the buffer;
        let checksum = u32::from_be_bytes(buf[0..=3].try_into().unwrap());
        let mut sum: u32 = 0;

        for byte in buf {
            sum += u32::from(*byte);
        }

        checksum == sum
    }
}
