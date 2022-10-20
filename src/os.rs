use linked_list_allocator::LockedHeap;

use crate::{exception::ExceptionSystem, interrupts};

pub enum SystemState {
    BeforeInit,
    BeforeMultitasking,
    BeginMultitasking,
    Up,
    Shutdown,
    Failed,
}

pub struct OS;

pub static MEM1_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[global_allocator]
pub static MEM2_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub static IPC_ALLOCATOR: LockedHeap = LockedHeap::empty();

extern "C" {
    pub type LinkerSymbol;
}

impl LinkerSymbol {
    pub fn as_ptr(&'static self) -> *const u8 {
        self as *const Self as *const u8
    }
    pub fn as_mut_ptr(&'static self) -> *mut u8 {
        self as *const Self as *mut Self as *mut u8
    }

    pub fn as_usize(&'static self) -> usize {
        self.as_ptr() as usize
    }
}

extern "C" {
    pub static ARENA_1_LO: LinkerSymbol;
}

pub const ARENA_1_HI: usize = 0x817FEFF0;
pub const ARENA_2_LO: usize = 0x90002000;
pub const ARENA_2_HI: usize = 0x933E0000;
pub const IPC_LO: usize = 0x93400000;
pub const IPC_HI: usize = 0x93400000;

impl OS {
    pub fn init() -> Self {
        let _isr = interrupts::disable();
        unsafe {
            low_mem_init();
            ipc_buffer_init();
        }

        ExceptionSystem::init();

        Self
    }
}

unsafe fn low_mem_init() {
    MEM1_ALLOCATOR
        .lock()
        .init(ARENA_1_LO.as_usize(), ARENA_1_HI - ARENA_1_LO.as_usize());
    MEM2_ALLOCATOR
        .lock()
        .init(ARENA_2_LO, ARENA_2_HI - ARENA_2_LO);
}

unsafe fn ipc_buffer_init() {
    IPC_ALLOCATOR.lock().init(IPC_LO, IPC_HI - IPC_LO);
}
