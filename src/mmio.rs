//TODO: Move `usize` to *mut T  to Option<NonNull<T>> wrapped in either Physical<T> or Virtual<T> or Cached<T>
// 0x0000_0000 = Physical<T>;
// 0x8000_0000 = Virtual<T>;
// 0xC000_0000 = Cached<T>;

pub mod ai;
pub mod dsp;
pub mod exi;
pub mod ipc;
//pub mod pe;
pub mod pi;
pub mod si;
pub mod vi;
