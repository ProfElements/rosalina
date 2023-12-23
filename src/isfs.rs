use alloc::vec;
use alloc::vec::Vec;

use crate::ios::{Error, FileAccessMode, Ios};

/// # Errors
/// any related Ios errors see `ios::Error`
pub fn read(path: impl AsRef<str>) -> Result<Vec<u8>, Error> {
    let mut file = open(path.as_ref())?;
    let size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut bytes = vec![0u8; size];
    if file.read(&mut bytes).is_ok() {
        Ok(bytes)
    } else {
        Err(file.read(&mut bytes).unwrap_err())
    }
}
/// # Errors
/// any related Ios errors see `ios::Error`
pub fn open(path: impl AsRef<str>) -> Result<Ios, Error> {
    let file = Ios::open(path.as_ref(), FileAccessMode::Read)?;
    Ok(file)
}
