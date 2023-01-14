use self::internal::{Error, IosFileDesc};

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

#[derive(Debug)]
pub enum FileError {
    NoSuchFile,
}

impl Ios {
    /// # Errors
    ///
    /// We return the `NoSuchFile` error when trying o access a file that doesn't exist
    pub fn open(file_name: &str, file_mode: FileAccessMode) -> Result<Self, Error> {
        let file = IosFileDesc::open(file_name, file_mode)?;
        Ok(Self { fd: file })
    }

    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        self.fd.read(buf).unwrap()
    }

    pub fn fd(&self) -> usize {
        self.fd.0.fd.try_into().unwrap()
    }
}

mod internal {

    use alloc::ffi::CString;

    use crate::ipc::{ios_close, ios_open, ios_read};

    use super::FileAccessMode;

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
            match ios_open(path.into(), mode.into()) {
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

        pub fn ioctl(&self, ioctl: usize, buf_in: &[u8], buf_out: &[u8]) -> Result<(), Error> {
            match ios_ioctl(ioctl, buf_in, buf_out) {
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
            buf_in: &[&[u8]],
            buf_out: &[&[u8]],
        ) -> Result<(), Error> {
            match ios_ioctlv(ioctl, buf_in, buf_out) {
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
        */
    }
}
