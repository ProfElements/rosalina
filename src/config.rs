use alloc::ffi::CString;

use crate::ios::{FileAccessMode, Ios};

pub struct Config {
    data: [u8; 0x4000],
    txt_data: [u8; 0x101],
}

impl Config {
    pub fn init() -> Self {
        let mut data = [0u8; 0x4000];
        let mut txt_data = [0u8; 0x101];

        let mut config = Ios::open(
            CString::new("/shared2/sys/SYSCONF").unwrap(),
            FileAccessMode::Read,
        )
        .unwrap();
        config.read(&mut data);

        let mut txt_fd = Ios::open(
            CString::new("/title/00000001/00000002/data/setting.txt").unwrap(),
            FileAccessMode::Read,
        )
        .unwrap();
        txt_fd.read(&mut txt_data);

        Self::decrypt_txt_buf(&mut txt_data);

        Self { data, txt_data }
    }

    fn decrypt_txt_buf(txt_buf: &mut [u8]) {
        let mut key: u32 = 0x73B5DBFA;

        for byte in txt_buf.iter_mut() {
            *byte ^= u8::try_from(key & 0xff).unwrap();
            key = (key << 1) | (key >> 31);
        }
    }

    pub const fn data(&self) -> &[u8] {
        &self.data
    }

    pub const fn txt_data(&self) -> &[u8] {
        &self.txt_data
    }
}
