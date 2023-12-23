use bytemuck::{bytes_of_mut, Pod};

use self::internal::IosFileDesc;

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
    fd: IosFileDesc,
}

#[repr(C)]
#[derive(Debug)]
pub enum Error {
    PermissionDenied,
    FileExists,
    InvalidArg,
    FileNotFound,
    ResourceBusy,
    Ecc,
    AllocFailed,
    CorruptedFile,
    TooManyFiles,
    PathToLong,
    FileisAlreadyOpen,
    DirectoryNotEmpty,
    Fatal,
}
#[repr(C)]
#[derive(Debug)]
pub struct Metadata {
    len: u32,
    seek_pos: u32,
}

impl Metadata {
    pub const fn new() -> Self {
        Self {
            len: 0,
            seek_pos: 0,
        }
    }

    pub fn len(&self) -> usize {
        usize::try_from(self.len).unwrap()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Ios {
    /// # Errors
    ///
    /// We return the `NoSuchFile` error when trying o access a file that doesn't exist
    pub fn open(file_name: &str, file_mode: FileAccessMode) -> Result<Self, Error> {
        let file = IosFileDesc::open(file_name, file_mode)?;
        Ok(Self { fd: file })
    }

    #[track_caller]
    /// # Errors
    /// See `Error`
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.fd.read(buf)
    }

    pub fn fd(&self) -> usize {
        self.fd.0.fd.try_into().unwrap()
    }
    /// # Errors
    /// Any related IOS errors see `Error`
    pub fn metadata(&self) -> Result<Metadata, Error> {
        let mut metadata = Metadata::new();
        self.fd.ioctl(11, &(), &mut metadata)?;
        crate::println!("{:?}", metadata);

        Ok(metadata)
    }

    pub fn ioctl<I, O>(&mut self, ioctl: u32, input: &I, output: &mut O) -> Result<(), Error> {
        self.fd.ioctl(ioctl.try_into().unwrap(), input, output)?;
        Ok(())
    }

    pub fn ioctlv(
        &mut self,
        ioctl: u32,
        count_in: u32,
        count_out: u32,
        iovecs_out: &mut [IoVec],
    ) -> Result<(), Error> {
        self.fd.ioctlv(
            ioctl.try_into().unwrap(),
            count_in.try_into().unwrap(),
            count_out.try_into().unwrap(),
            iovecs_out,
        )?;
        Ok(())
    }
}

pub struct IoVec<'a> {
    pub ptr: &'a mut u8,
    pub len: usize,
}

impl<'a> IoVec<'a> {
    pub fn new<T: Pod>(data: &'a mut T) -> Self {
        let bytes = bytes_of_mut(data);
        let bytes_len = bytes.len();
        let bytes_ptr = unsafe { bytes.as_mut_ptr().as_mut().unwrap() };
        Self {
            ptr: bytes_ptr,
            len: bytes_len,
        }
    }
}

mod internal {

    use alloc::ffi::CString;

    use crate::ipc::{ios_close, ios_ioctl, ios_ioctlv, ios_open, ios_read};

    use super::{Error, FileAccessMode, IoVec};

    type IosRawFd = isize;

    pub struct OwnedIosFd {
        pub(crate) fd: IosRawFd,
    }

    impl Drop for OwnedIosFd {
        fn drop(&mut self) {
            ios_close(self.fd);
        }
    }

    pub struct IosFileDesc(pub(crate) OwnedIosFd);

    /*
    pub enum FileSeek {
        Start(usize),
        End(usize),
        Curr(usize),
    }
    */

    impl IosFileDesc {
        pub fn open(path: &str, mode: FileAccessMode) -> Result<Self, Error> {
            let path = CString::new(path).unwrap();
            match ios_open(path, mode.into()) {
                -1 | -102 => Err(Error::PermissionDenied),
                -2 | -105 => Err(Error::FileExists),
                -4 | -101 => Err(Error::InvalidArg),
                -6 | -106 => Err(Error::FileNotFound),
                -8 | -118 => Err(Error::ResourceBusy),
                -12 | -114 => Err(Error::Ecc),
                -103 => Err(Error::CorruptedFile),
                -107 | -109 => Err(Error::TooManyFiles),
                -22 | -108 => Err(Error::AllocFailed),
                -110 | -116 => Err(Error::PathToLong),
                -111 => Err(Error::FileisAlreadyOpen),
                -115 => Err(Error::DirectoryNotEmpty),
                -119 => Err(Error::Fatal),
                val => Ok(Self(OwnedIosFd { fd: val })),
            }
        }

        pub fn read(&self, buf: &mut [u8]) -> Result<usize, Error> {
            match ios_read(self.0.fd, buf) {
                -1 | -102 => Err(Error::PermissionDenied),
                -2 | -105 => Err(Error::FileExists),
                -4 | -101 => Err(Error::InvalidArg),
                -6 | -106 => Err(Error::FileNotFound),
                -8 | -118 => Err(Error::ResourceBusy),
                -12 | -114 => Err(Error::Ecc),
                -103 => Err(Error::CorruptedFile),
                -107 | -109 => Err(Error::TooManyFiles),
                -22 | -108 => Err(Error::AllocFailed),
                -110 | -116 => Err(Error::PathToLong),
                -111 => Err(Error::FileisAlreadyOpen),
                -115 => Err(Error::DirectoryNotEmpty),
                -119 => Err(Error::Fatal),
                val => Ok(val.try_into().unwrap()),
            }
        }
        /*
                pub fn write(&self, buf: &[u8]) -> Result<usize, Error> {
                    match ios_write(self.0.fd, buf) {
                        -1 | -102 => Err(Error::PermissionDenied),
                        -2 | -105 => Err(Error::FileExists),
                        -4 | -101 => Err(Error::InvalidArg),
                        -6 | -106 => Err(Error::FileNotFound),
                        -8 | -118 => Err(Error::ResourceBusy),
                        -12 | -114 => Err(Error::Ecc),
                        -103 => Err(Error::CorruptedFile),
                        -107 | -109 => Err(Error::TooManyFiles),
                        -22 | -108 => Err(Error::AllocFailed),
                        -110 | -116 => Err(Error::PathToLong),
                        -111 => Err(Error::FileisAlreadyOpen),
                        -115 => Err(Error::DirectoryNotEmpty),
                        -119 => Err(Error::Fatal),
                        val => Ok(val.try_into().unwrap()),
                    }
                }

                pub fn seek(&self, pos: FileSeek) -> Result<usize, Error> {
                    match ios_seek(self.0.fd, pos.into()) {
                        -1 | -102 => Err(Error::PermissionDenied),
                        -2 | -105 => Err(Error::FileExists),
                        -4 | -101 => Err(Error::InvalidArg),
                        -6 | -106 => Err(Error::FileNotFound),
                        -8 | -118 => Err(Error::ResourceBusy),
                        -12 | -114 => Err(Error::Ecc),
                        -103 => Err(Error::CorruptedFile),
                        -107 | -109 => Err(Error::TooManyFiles),
                        -22 | -108 => Err(Error::AllocFailed),
                        -110 | -116 => Err(Error::PathToLong),
                        -111 => Err(Error::FileisAlreadyOpen),
                        -115 => Err(Error::DirectoryNotEmpty),
                        -119 => Err(Error::Fatal),
                        val => Ok(val.try_into().unwrap()),
                    }
                }
        */
        pub fn ioctl<I, O>(&self, ioctl: usize, buf_in: &I, buf_out: &mut O) -> Result<(), Error> {
            match ios_ioctl(self.0.fd, ioctl, buf_in, buf_out) {
                -1 | -102 => Err(Error::PermissionDenied),
                -2 | -105 => Err(Error::FileExists),
                -4 | -101 => Err(Error::InvalidArg),
                -6 | -106 => Err(Error::FileNotFound),
                -8 | -118 => Err(Error::ResourceBusy),
                -12 | -114 => Err(Error::Ecc),
                -103 => Err(Error::CorruptedFile),
                -107 | -109 => Err(Error::TooManyFiles),
                -22 | -108 => Err(Error::AllocFailed),
                -110 | -116 => Err(Error::PathToLong),
                -111 => Err(Error::FileisAlreadyOpen),
                -115 => Err(Error::DirectoryNotEmpty),
                -119 => Err(Error::Fatal),
                _ => Ok(()),
            }
        }

        pub fn ioctlv(
            &self,
            ioctl: usize,
            count_in: usize,
            count_out: usize,
            buf_out: &mut [IoVec],
        ) -> Result<(), Error> {
            match ios_ioctlv(self.0.fd, ioctl, count_in, count_out, buf_out) {
                -1 | -102 => Err(Error::PermissionDenied),
                -2 | -105 => Err(Error::FileExists),
                -4 | -101 => Err(Error::InvalidArg),
                -6 | -106 => Err(Error::FileNotFound),
                -8 | -118 => Err(Error::ResourceBusy),
                -12 | -114 => Err(Error::Ecc),
                -103 => Err(Error::CorruptedFile),
                -107 | -109 => Err(Error::TooManyFiles),
                -22 | -108 => Err(Error::AllocFailed),
                -110 | -116 => Err(Error::PathToLong),
                -111 => Err(Error::FileisAlreadyOpen),
                -115 => Err(Error::DirectoryNotEmpty),
                -119 => Err(Error::Fatal),
                _ => Ok(()),
            }
        }
    }
}
