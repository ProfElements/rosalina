use voladdress::{Safe, VolAddress};

pub struct WriteGatherPipe;

const WRITER_GATHER_PIPE_U8: VolAddress<u8, (), Safe> = unsafe { VolAddress::new(0xCC00_8000) };
const WRITER_GATHER_PIPE_U32: VolAddress<u32, (), Safe> = unsafe { VolAddress::new(0xCC00_8000) };
const WRITE_GATHER_PIPE_F32: VolAddress<f32, (), Safe> = unsafe { VolAddress::new(0xCC00_8000) };

impl Default for WriteGatherPipe {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteGatherPipe {
    pub fn new() -> Self {
        Self
    }

    pub fn write_u8(&mut self, byte: u8) -> &mut Self {
        WRITER_GATHER_PIPE_U8.write(byte);
        self
    }

    pub fn write_u32(&mut self, bytes: u32) -> &mut Self {
        WRITER_GATHER_PIPE_U32.write(bytes);
        self
    }

    pub fn write_f32(&mut self, bytes: f32) -> &mut Self {
        WRITE_GATHER_PIPE_F32.write(bytes);
        self
    }

    pub fn write_bp_reg(&mut self, bytes: u32) -> &mut Self {
        WRITER_GATHER_PIPE_U8.write(0x61);
        WRITER_GATHER_PIPE_U32.write(bytes);
        self
    }

    pub fn write_cp_reg(&mut self, reg: u8, bytes: u32) -> &mut Self {
        WRITER_GATHER_PIPE_U8.write(0x08);
        WRITER_GATHER_PIPE_U8.write(reg);
        WRITER_GATHER_PIPE_U32.write(bytes);
        self
    }

    pub fn write_xf_reg(&mut self, reg: u32, bytes: u32) -> &mut Self {
        WRITER_GATHER_PIPE_U8.write(0x10);
        WRITER_GATHER_PIPE_U32.write(reg);
        WRITER_GATHER_PIPE_U32.write(bytes);
        self
    }
}
