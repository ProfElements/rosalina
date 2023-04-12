//TODO: Move `usize` to *mut T  to Option<NonNull<T>> wrapped in either Physical<T> or Virtual<T> or Cached<T>
// 0x0000_0000 = Physical<T>;
// 0x8000_0000 = Virtual<T>;
// 0xC000_0000 = Uncached<T>;

pub mod ai;
pub mod dsp;
pub mod exi;
pub mod ipc;
//pub mod pe;
pub mod cp;
pub mod pi;
pub mod si;
pub mod vi;

pub struct Physical<T: ?Sized> {
    ptr: *mut T,
}

impl<T: ?Sized> Physical<T> {
    pub fn new(ptr: *mut T) -> Self {
        let addr = ptr.addr();

        match addr {
            0x8000_0000..=0x817F_FFFF | 0x9000_0000..=0x93FF_FFFF => Self {
                ptr: ptr.map_addr(|addr| addr - 0x8000_0000),
            },

            0xC000_0000..=0xC17F_FFFF | 0xD000_0000..=0xD3FF_FFFF => Self {
                ptr: ptr.map_addr(|addr| addr - 0xC000_0000),
            },

            0x0000_0000..=0x017F_FFFF | 0x1000_0000..=0x13FF_FFFF => Self { ptr },
            _ => panic!("something wrong happened"),
        }
    }

    pub fn addr(self) -> usize {
        self.ptr.addr()
    }
}
