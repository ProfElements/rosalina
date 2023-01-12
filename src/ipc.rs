use core::{
    fmt::Write,
    ptr::{from_exposed_addr, from_exposed_addr_mut},
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::{boxed::Box, ffi::CString};
use voladdress::{Safe, VolAddress, VolBlock};

use crate::{
    cache::{dc_flush_range, dc_invalidate_range},
    interrupts::Interrupt,
    mmio::pi::{InterruptMask, Mask},
    DOLPHIN_HLE,
};

static REQ_MAGIC: AtomicUsize = AtomicUsize::new(0);
pub const IOS_COUNT: usize = 32;

pub const BASE: usize = 0xCD00_0000;
pub const IPC_VOLBLOCK: VolBlock<u32, Safe, Safe, 0x80> = unsafe { VolBlock::new(BASE) };

pub const PPC_CTRL: VolAddress<usize, Safe, Safe> = unsafe { VolAddress::new(BASE + 0x4) };
pub const PPC_MSG: VolAddress<usize, Safe, Safe> = unsafe { VolAddress::new(BASE) };

pub struct Ipc;

impl Ipc {
    pub fn init() -> Self {
        REQ_MAGIC.store(0xDEEDBEEF, Ordering::Relaxed);

        Interrupt::set_interrupt_handler(Interrupt::InterprocessControl, |_| {
            let reg = PPC_CTRL.read();
            unsafe { write!(DOLPHIN_HLE, "IPC HIT").unwrap() };
            if reg & 0x0014 == 0x0014 {
                let request: *const IpcRequest = from_exposed_addr(
                    usize::try_from(IPC_VOLBLOCK.get(2).unwrap().read()).unwrap(),
                );
                if request.is_null() {
                    return Err("Request is null");
                }
                // ACKCKCKCKC
                PPC_CTRL.write((reg & 0x30) | 0x04);
                IPC_VOLBLOCK.get(48 >> 2).unwrap().write(0x40000000);

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

                PPC_CTRL.write((reg & 0x30) | 0x8);
            }

            if reg & 0x0022 == 0x0022 {
                PPC_CTRL.write(reg & 0x30 | 0x2);
                IPC_VOLBLOCK.get(48 >> 2).unwrap().write(0x40000000);

                PPC_CTRL.write(reg & 0x30 | 0x8);
            }

            Ok(())
        });

        InterruptMask::read()
            .with_interprocess_control(Mask::Enabled)
            .write();

        PPC_CTRL.write(56);

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
    PPC_MSG.write(request.map_addr(|addr| addr - 0x8000_0000).addr());
    PPC_CTRL.write((PPC_CTRL.read() & 0x30) | 0x01);
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
    PPC_MSG.write(request.map_addr(|addr| addr - 0x8000_0000).addr());
    PPC_CTRL.write((PPC_CTRL.read() & 0x30) | 0x01);
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
    PPC_MSG.write(request.map_addr(|addr| addr - 0x8000_0000).addr());
    PPC_CTRL.write((PPC_CTRL.read() & 0x30) | 0x01);
}

pub fn ios_read(fd: isize, buf: &mut [u8]) -> usize {
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
    PPC_MSG.write(request.map_addr(|addr| addr - 0x8000_0000).addr());
    PPC_CTRL.write((PPC_CTRL.read() & 0x30) | 0x01);
    unsafe { *request }.result.try_into().unwrap()
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
