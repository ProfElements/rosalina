use alloc::ffi::CString;

use crate::ipc::{ios_close, ios_open, ios_read};

pub enum FileAccessMode {
    None,
    Read,
    Write,
    ReadWrite,
}

impl From<FileAccessMode> for usize {
    fn from(value: FileAccessMode) -> Self {
        match value {
            FileAccessMode::None => 0,
            FileAccessMode::Read => 1,
            FileAccessMode::Write => 2,
            FileAccessMode::ReadWrite => 3,
        }
    }
}

pub struct Ios {
    fd: isize,
}

#[derive(Debug)]
pub enum FileError {
    NoSuchFile,
}

impl Ios {
    /// # Errors
    ///
    /// We return the `NoSuchFile` error when trying o access a file that doesn't exist
    pub fn open(file_name: CString, file_mode: FileAccessMode) -> Result<Self, FileError> {
        match ios_open(file_name, file_mode.into()) {
            -106 => Err(FileError::NoSuchFile),
            val => Ok(Self { fd: val }),
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        ios_read(self.fd, buf)
    }
}

impl Drop for Ios {
    fn drop(&mut self) {
        ios_close(self.fd);
    }
}
