use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::vec::Vec;

use crate::ipc::rev2::{IpcAccessMode, IpcError};
use crate::{ios::Metadata, ipc::rev2::IpcRequest};

/// # Errors
/// any related Ios errors see `ios::Error`
pub fn read(path: impl AsRef<str>) -> Result<Vec<u8>, IpcError> {
    let file = open(path.as_ref())?;

    let mut req = IpcRequest::ioctl(file, 11, Box::new(()), Box::new(Metadata::new())).send()?;

    let metadata = req.take_output::<Metadata>().unwrap();
    let size = if metadata.len() > 0 {
        metadata.len()
    } else {
        0
    };

    IpcRequest::read(file, Vec::with_capacity(size))
        .send()?
        .take_buf()
        .map_or_else(
            || Err(IpcError::Other("Unable to take buf from ipc request")),
            Ok,
        )
}

/// # Errors
/// any related Ios errors see `ios::Error`
pub fn open(path: impl AsRef<str>) -> Result<u32, IpcError> {
    IpcRequest::open(
        CString::new(path.as_ref()).unwrap(),
        IpcAccessMode::ReadWrite,
    )
    .send()
    .map(|req| req.ret.try_into().unwrap())
}
