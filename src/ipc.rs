use core::{
    fmt::Write,
    ptr::{from_exposed_addr, from_exposed_addr_mut},
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::{boxed::Box, ffi::CString};

use crate::{
    cache::{dc_flush_range, dc_invalidate_range},
    interrupts::Interrupt,
    mmio::{
        ipc::{IpcControl, IpcInterruptFlags, IpcRequestAddr},
        pi::{InterruptMask, InterruptState, Mask},
    },
    DOLPHIN_HLE,
};

static REQ_MAGIC: AtomicUsize = AtomicUsize::new(0);
pub const IOS_COUNT: usize = 32;

//TODO:
// Change callback_addr + callback_data = Option<Box<dyn Fn(*mut ())>>;
// Make IpcCommand repr(u32) and just set it in IpcRequest
// Somehow not leak the data of IpcRequest longer the the reply handler

pub struct Ipc;

impl Ipc {
    pub fn init() -> Self {
        REQ_MAGIC.store(0xDEEDBEEF, Ordering::Relaxed);

        Interrupt::set_interrupt_handler(Interrupt::InterprocessControl, |_| {
            let ctrl = IpcControl::read_ppc();
            unsafe { write!(DOLPHIN_HLE, "IPC HIT",).unwrap() };
            if ctrl.y1() && ctrl.ix1() {
                unsafe { write!(DOLPHIN_HLE, "GOT TO REQUEST",).unwrap() };
                let request: *const IpcRequest =
                    from_exposed_addr::<IpcRequest>(IpcRequestAddr::read_arm().addr());
                if request.is_null() {
                    return Err("Request is null");
                }
                // ACKCKCKCKC

                IpcControl::new()
                    .with_ix1(ctrl.ix1())
                    .with_ix2(ctrl.ix2())
                    .with_y1(true)
                    .write_ppc();

                IpcInterruptFlags::new()
                    .with_ipc_interupt(InterruptState::Happened)
                    .write_ppc();

                let request = request.map_addr(|addr| addr + 0x8000_0000);
                dc_invalidate_range(
                    request.cast::<u8>().cast_mut(),
                    core::mem::size_of::<IpcRequest>(),
                );

                let req_ref = unsafe { request.as_ref().unwrap() };
                if req_ref.magic == REQ_MAGIC.load(Ordering::Relaxed) {
                    if let Ok(cmd) = IPCCommand::try_from(req_ref.cmd) {
                        match cmd {
                            IPCCommand::Open => todo!(),
                            IPCCommand::Close => todo!(),
                            IPCCommand::Read => todo!(),
                            IPCCommand::Write => todo!(),
                            IPCCommand::Seek => todo!(),
                            IPCCommand::Ioctl => todo!(),
                            IPCCommand::Ioctlv => todo!(),
                        }
                    }
                }

                if unsafe { *request }.callback_addr.is_some() {
                    unsafe {
                        let ptr: Box<dyn Fn(*mut ())> =
                            Box::from_raw((*request).callback_addr.unwrap());

                        ptr(from_exposed_addr_mut((*request).callback_data));
                    }
                }

                IpcControl::new()
                    .with_ix1(ctrl.ix1())
                    .with_ix2(ctrl.ix2())
                    .with_y2(true)
                    .write_ppc();
            }

            if ctrl.x2() && ctrl.ix2() {
                IpcControl::new()
                    .with_ix1(ctrl.ix1())
                    .with_ix2(ctrl.ix2())
                    .with_x2(true)
                    .write_ppc();

                IpcInterruptFlags::new()
                    .with_ipc_interupt(InterruptState::Happened)
                    .write_ppc();

                IpcControl::new()
                    .with_ix1(ctrl.ix1())
                    .with_ix2(ctrl.ix2())
                    .with_y2(true)
                    .write_ppc();
            }

            Ok(())
        });

        InterruptMask::read()
            .with_interprocess_control(Mask::Enabled)
            .write();

        IpcControl::new()
            .with_y2(true)
            .with_ix1(true)
            .with_ix2(true)
            .write_ppc();
        Self
    }
}

#[repr(C, align(32))]
#[derive(Debug, Copy, Clone)]
pub struct IpcRequest {
    cmd: usize,
    result: isize,
    fd: isize,
    args: [usize; 5],
    callback_addr: Option<*mut dyn Fn(*mut ())>,
    callback_data: usize,
    relauch: usize,
    queue: usize,
    magic: usize,
    pad: [u8; 12],
}

#[derive(PartialEq, Eq)]
#[repr(C)]
pub enum IPCCommand {
    Open,
    Close,
    Read,
    Write,
    Seek,
    Ioctl,
    Ioctlv,
}

#[derive(Debug)]
pub struct InvalidIpcCommandError;

impl TryFrom<usize> for IPCCommand {
    type Error = InvalidIpcCommandError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0x1 => Ok(Self::Open),
            0x2 => Ok(Self::Close),
            0x3 => Ok(Self::Read),
            0x4 => Ok(Self::Write),
            0x5 => Ok(Self::Seek),

            0x6 => Ok(Self::Ioctl),
            0x7 => Ok(Self::Ioctlv),
            _ => Err(InvalidIpcCommandError),
        }
    }
}

impl From<IPCCommand> for usize {
    fn from(value: IPCCommand) -> Self {
        match value {
            IPCCommand::Open => 1,
            IPCCommand::Close => 2,
            IPCCommand::Read => 3,
            IPCCommand::Write => 4,
            IPCCommand::Seek => 5,
            IPCCommand::Ioctl => 6,
            IPCCommand::Ioctlv => 7,
        }
    }
}

pub fn ios_open(file_path: CString, mode: usize) -> isize {
    let request = Box::leak(Box::new(IpcRequest::new()));

    request.cmd = usize::from(IPCCommand::Open);
    request.relauch = 0x0;
    let len = file_path.as_bytes_with_nul().len();
    let raw = file_path.into_raw();
    dc_flush_range(raw.cast(), len);

    request.args[0] = raw.map_addr(|addr| addr - 0x8000_0000).addr();
    request.args[1] = mode;

    let request: *mut IpcRequest = request;
    dc_flush_range(request.cast(), core::mem::size_of::<IpcRequest>());
    IpcRequestAddr::new()
        .with_addr(request.map_addr(|addr| addr - 0x8000_0000).addr())
        .write_ppc();

    IpcControl::new()
        .with_ix1(IpcControl::read_ppc().ix1())
        .with_ix2(IpcControl::read_ppc().ix2())
        .with_x1(true)
        .write_ppc();
    unsafe {
        write!(DOLPHIN_HLE, "{:?}", *request).unwrap();

        let _str = CString::from_raw(raw);
        (*request).result
    }
}
pub fn ios_close(fd: isize) {
    let request = Box::leak(Box::new(IpcRequest::new()));
    request.magic = REQ_MAGIC.load(Ordering::Relaxed);
    request.cmd = usize::from(IPCCommand::Close);
    request.fd = fd;
    request.relauch = 0;

    let request: *mut IpcRequest = request;
    dc_flush_range(request.cast(), core::mem::size_of::<IpcRequest>());
    IpcRequestAddr::new()
        .with_addr(request.map_addr(|addr| addr - 0x8000_0000).addr())
        .write_ppc();

    IpcControl::new()
        .with_ix1(IpcControl::read_ppc().ix1())
        .with_ix2(IpcControl::read_ppc().ix2())
        .with_x1(true)
        .write_ppc();
}

pub fn ios_ioctl_async<T>(
    fd: isize,
    ioctl: usize,
    buffer_in: &[u8],
    buffer_out: &[u8],
    func: Option<impl Fn(*mut ()) + 'static>,
    func_data: Option<&'static mut T>,
) {
    let request = Box::leak(Box::new(IpcRequest::new()));
    request.cmd = usize::from(IPCCommand::Ioctl);
    request.fd = fd;
    if let Some(func) = func {
        let x: *mut dyn Fn(*mut ()) = Box::leak(Box::new(func));
        request.callback_addr = Some(x);
    }

    if let Some(data) = func_data {
        request.callback_data = core::ptr::from_mut(data).cast::<()>().expose_addr();
    }

    request.args[0] = ioctl;
    request.args[1] = buffer_in
        .as_ptr()
        .map_addr(|addr| addr - 0x8000_0000)
        .addr();
    request.args[2] = buffer_in.len();
    request.args[3] = buffer_out
        .as_ptr()
        .map_addr(|addr| addr - 0x8000_0000)
        .addr();
    request.args[4] = buffer_out.len();
    request.relauch = 0x0;
    dc_flush_range(buffer_out.as_ptr(), buffer_out.len());
    dc_flush_range(buffer_in.as_ptr(), buffer_in.len());

    let request: *mut IpcRequest = request;
    dc_flush_range(request.cast(), core::mem::size_of::<IpcRequest>());

    IpcRequestAddr::new()
        .with_addr(request.map_addr(|addr| addr - 0x8000_0000).addr())
        .write_ppc();

    IpcControl::new()
        .with_ix1(IpcControl::read_ppc().ix1())
        .with_ix2(IpcControl::read_ppc().ix2())
        .with_x1(true)
        .write_ppc();
}

pub fn ios_read(fd: isize, buf: &mut [u8]) -> isize {
    let request = Box::leak(Box::new(IpcRequest::new()));
    request.magic = REQ_MAGIC.load(Ordering::Relaxed);
    request.cmd = usize::from(IPCCommand::Read);
    request.fd = fd;
    request.relauch = 0;

    dc_invalidate_range(buf.as_mut_ptr(), buf.len());
    request.args[0] = buf
        .as_mut_ptr()
        .map_addr(|addr| addr - 0x8000_0000)
        .expose_addr();
    request.args[1] = buf.len();

    let request: *mut IpcRequest = request;
    dc_flush_range(request.cast(), core::mem::size_of::<IpcRequest>());
    IpcRequestAddr::new()
        .with_addr(request.map_addr(|addr| addr - 0x8000_0000).addr())
        .write_ppc();

    IpcControl::new()
        .with_ix1(IpcControl::read_ppc().ix1())
        .with_ix2(IpcControl::read_ppc().ix2())
        .with_x1(true)
        .write_ppc();
    unsafe { *request }.result
}

impl IpcRequest {
    pub fn new() -> Self {
        Self {
            cmd: 0,
            result: 0,
            fd: 0,
            args: [0, 0, 0, 0, 0],
            callback_addr: None,
            callback_data: 0,
            relauch: 0,
            queue: 0,
            magic: 0,
            pad: [0; 12],
        }
    }
}

impl Default for IpcRequest {
    fn default() -> Self {
        Self::new()
    }
}
