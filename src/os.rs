use core::{
    mem::ManuallyDrop,
    ptr::from_exposed_addr_mut,
    sync::atomic::{AtomicUsize, Ordering},
};

use bit_field::BitField;
use linked_list_allocator::LockedHeap;

use crate::{
    clock::{self, Instant, TB_TIMER_CLOCK},
    config::Config,
    exception::Exception,
    exi::ExternalInterface,
    interrupts,
    ios::{FileAccessMode, Ios},
    ipc::{ios_ioctl_async, Ipc},
    mmio::{
        dsp::{
            AddrHi, AddrLo, AramDmaCountHi, AramDmaCountLo, AramSize, DmaType, DspControl, Halt,
            MailboxHi, MailboxLow,
        },
        exi::DmaStart,
        pi::InterruptState,
        vi::Reset,
    },
    println,
    si::SerialInterface,
    sram::Sram,
    wii::Wii,
};

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
    pub const fn as_self_ptr(&'static self) -> *const Self {
        self
    }

    pub const fn as_ptr(&'static self) -> *const u8 {
        self.as_self_ptr().cast::<u8>()
    }

    pub const fn as_mut_ptr(&'static self) -> *mut u8 {
        self.as_self_ptr().cast_mut().cast::<u8>()
    }

    pub fn as_usize(&'static self) -> usize {
        self.as_ptr().addr()
    }
}

extern "C" {
    pub static ARENA_1_LO: LinkerSymbol;
}

pub const ARENA_1_HI: usize = 0x817FEFF0;
pub const ARENA_2_LO: usize = 0x90002000;
pub const ARENA_2_HI: usize = 0x933E0000;
pub const IPC_LO: usize = 0x933e0000;
pub const IPC_HI: usize = 0x93400000;

impl OS {
    pub fn init() -> Self {
        interrupts::disable();
        unsafe {
            low_mem_init();
            ipc_buffer_init();
        }

        Exception::init();
        ExternalInterface::init();
        clock::set_time(u64::from(ExternalInterface::get_rtc()) * (TB_TIMER_CLOCK * 1000u64));
        Sram::init();
        SerialInterface::init();
        Ipc::init();

        unsafe { dsp_bootstrap() }

        interrupts::enable();

        let _ = ManuallyDrop::new(Ios::open("/dev/es", FileAccessMode::None).unwrap());
        let _ = Ios::open("/dev/stm/immediate", FileAccessMode::None).unwrap();
        let event_hook = Ios::open("/dev/stm/eventhook", FileAccessMode::None).unwrap();

        EVT_FD.store(event_hook.fd(), Ordering::Relaxed);

        ios_ioctl_async(
            event_hook.fd().try_into().unwrap(),
            0x1000,
            &[0u8; 0x20],
            &[0u8; 0x20],
            Some(stm_event_handler),
            None::<&mut ()>,
        );

        let _ = Config::init();
        let _ = Wii::init();
        Self
    }
}

static EVT_FD: AtomicUsize = AtomicUsize::new(0);

fn stm_event_handler(_data: *mut ()) {
    ios_ioctl_async(
        EVT_FD.load(Ordering::Relaxed).try_into().unwrap(),
        0x1000,
        &[0u8; 0x20],
        &[0u8; 0x20],
        Some(stm_event_handler),
        None::<&mut ()>,
    );
}

#[repr(align(32))]
pub struct Align32<T>(pub T);

unsafe fn low_mem_init() {
    MEM1_ALLOCATOR
        .lock()
        .init(ARENA_1_LO.as_mut_ptr(), ARENA_1_HI - ARENA_1_LO.as_usize());
    MEM2_ALLOCATOR
        .lock()
        .init(from_exposed_addr_mut(ARENA_2_LO), ARENA_2_HI - ARENA_2_LO);
}

unsafe fn ipc_buffer_init() {
    IPC_ALLOCATOR
        .lock()
        .init(from_exposed_addr_mut(IPC_LO), IPC_HI - IPC_LO);
}

unsafe fn dsp_bootstrap() {
    core::ptr::copy_nonoverlapping(
        from_exposed_addr_mut::<u32>(0x8100_0000),
        from_exposed_addr_mut::<u32>(ARENA_1_HI - 128),
        DSP_INIT_CODE.len(),
    );
    core::ptr::copy_nonoverlapping(
        DSP_INIT_CODE.as_ptr(),
        from_exposed_addr_mut(0x8100_0000),
        DSP_INIT_CODE.len(),
    );

    AramSize::new().with_size(67).write();

    DspControl::new()
        .with_reset(Reset::Reset)
        .with_dsp_interrupt(InterruptState::Happened)
        .with_aram_interrupt(InterruptState::Happened)
        .with_dma_interrupt(InterruptState::Happened)
        .with_halt(Halt::Halted)
        .write();

    DspControl::read().with_reset(Reset::Reset).write();

    while DspControl::read().reset() == Reset::Reset {
        core::hint::spin_loop();
    }

    MailboxHi::from(0x0).write_dsp();
    let _ = MailboxLow::read_cpu();

    while MailboxHi::read_cpu().mailbox_status() == DmaStart::Start {
        core::hint::spin_loop();
    }

    let main_mem_addr: usize = 0x0100_0000;

    AddrHi::new()
        .with_addr_high(main_mem_addr.get_bits(16..=31).try_into().unwrap())
        .write_main_mem();
    AddrLo::new()
        .with_addr_low(main_mem_addr.get_bits(0..=15).try_into().unwrap())
        .write_main_mem();

    AddrHi::new().write_audio_mem();
    AddrLo::new().write_audio_mem();

    AramDmaCountLo::new().with_count_low(32).write();
    AramDmaCountHi::new().with_dma_type(DmaType::Write).write();

    while DspControl::read().aram_interrupt() == InterruptState::Idle {
        core::hint::spin_loop();
    }

    DspControl::read()
        .with_aram_interrupt(InterruptState::Happened)
        .write();

    let now = Instant::now();
    while Instant::now().ticks - now.ticks < 2173 {
        core::hint::spin_loop();
    }

    AddrHi::new()
        .with_addr_high(main_mem_addr.get_bits(16..=31).try_into().unwrap())
        .write_main_mem();
    AddrLo::new()
        .with_addr_low(main_mem_addr.get_bits(0..=15).try_into().unwrap())
        .write_main_mem();

    AddrHi::new().write_audio_mem();
    AddrLo::new().write_audio_mem();

    AramDmaCountLo::new().with_count_low(32).write();
    AramDmaCountHi::new().with_dma_type(DmaType::Write).write();

    while DspControl::read().aram_interrupt() == InterruptState::Idle {
        core::hint::spin_loop();
    }

    DspControl::read()
        .with_aram_interrupt(InterruptState::Happened)
        .write();

    DspControl::read().with_reset(Reset::NoReset).write();

    while DspControl::read().inited1() {
        core::hint::spin_loop();
    }

    DspControl::read().with_halt(Halt::Idle).write();

    while MailboxHi::read_cpu().mailbox_status() == DmaStart::Idle {
        core::hint::spin_loop();
    }

    DspControl::read().with_halt(Halt::Halted).write();

    DspControl::new()
        .with_halt(Halt::Halted)
        .with_reset(Reset::Reset)
        .with_aram_interrupt(InterruptState::Happened)
        .with_dma_interrupt(InterruptState::Happened)
        .with_dsp_interrupt(InterruptState::Happened)
        .write();

    DspControl::read().with_reset(Reset::Reset).write();

    while DspControl::read().reset() == Reset::Reset {
        core::hint::spin_loop();
    }

    core::ptr::copy_nonoverlapping(
        from_exposed_addr_mut::<u32>(0x8100_0000),
        from_exposed_addr_mut::<u32>(ARENA_1_HI - 128),
        DSP_INIT_CODE.len(),
    );
}

static DSP_INIT_CODE: [u32; 32] = [
    0x029F0010, 0x029F0033, 0x029F0034, 0x029F0035, 0x029F0036, 0x029F0037, 0x029F0038, 0x029F0039,
    0x12061203, 0x12041205, 0x00808000, 0x0088FFFF, 0x00841000, 0x0064001D, 0x02180000, 0x81001C1E,
    0x00441B1E, 0x00840800, 0x00640027, 0x191E0000, 0x00DEFFFC, 0x02A08000, 0x029C0028, 0x16FC0054,
    0x16FD4348, 0x002102FF, 0x02FF02FF, 0x02FF02FF, 0x02FF02FF, 0x00000000, 0x00000000, 0x00000000,
];
