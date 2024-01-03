use bytemuck::{bytes_of_mut, Pod};
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
