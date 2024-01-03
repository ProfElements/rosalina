use core::ptr::from_exposed_addr_mut;

use alloc::boxed::Box;

use crate::{
    interrupts::Interrupt,
    mmio::pi::{InterruptCause, InterruptMask, InterruptState, Mask},
};

use self::rev2::{IpcMessageAddress, PpcIpcControl};
//TODO:
// Change callback_addr + callback_data = Option<Box<dyn Fn(*mut ())>>;
// Make IpcCommand repr(u32) and just set it in IpcRequest
// Somehow not leak the data of IpcRequest longer the the reply handler

pub struct Ipc;

impl Ipc {
    pub fn init() -> Self {
        Interrupt::set_interrupt_handler(Interrupt::InterprocessControl, |_| {
            if PpcIpcControl::read().acknowledge() {
                PpcIpcControl::new().with_acknowledge(true).write();
            }

            if PpcIpcControl::read().reply() {
                let req_addr = IpcMessageAddress::read_arm().address().try_into().unwrap();
                let mut req_ptr = from_exposed_addr_mut::<rev2::IpcRequest>(req_addr);
                req_ptr = req_ptr.map_addr(|addr| addr | 0x8000_0000);
                let req = unsafe { Box::from_raw(req_ptr) };

                if let Some(cb) = req.callback {
                    let cb = unsafe {
                        Box::from_raw(Box::into_raw(cb).map_addr(|addr| addr | 0x8000_0000))
                    };
                    crate::println!("Got callback");
                    cb(req_ptr);
                }

                PpcIpcControl::new().with_reply(true).write();
                InterruptCause::new()
                    .with_interprocess_control(InterruptState::Happened)
                    .write();
            }

            Ok(())
        });

        InterruptMask::read()
            .with_interprocess_control(Mask::Enabled)
            .write();

        Self
    }
}

pub mod rev2 {
    use core::{convert::TryInto, fmt::Debug, ptr::from_exposed_addr_mut};

    use alloc::{boxed::Box, ffi::CString, vec::Vec};

    use crate::{cache::dc_flush_range, ios::IoVec};

    //bits 0..=31 = physical address of ipc request
    pub const HW_IPC_PPCMSG: usize = 0xCD00_0000usize; //from_exposed_addr_mut(0xCD00_0000);

    //bit 0 = X1 | Execute IPC request
    //bit 1 = Y2 | Acknowledge IPC request
    //bit 2 = Y1 | IPC request reply available
    //bit 3 = X2 | Relaunch IPC
    //bit 4 = IY1 | IPC request reply send out IPC interrupt
    //bit 5 = IY2 | IPC request acknowledge sends out IPC interrupt
    pub const HW_IPC_PPCCTRL: usize = 0xCD00_0004usize; //from_exposed_addr_mut(0xCD00_0004);

    //bits 0..=31 = physical address of ipc request
    pub const HW_IPC_ARMMSG: usize = 0xCD00_0008usize; //from_exposed_addr_mut(0xCD00_0008);

    //bit 0 = Y1 | IPC request reply available
    //bit 1 = X2 | Relauch IPC
    //bit 2 = X1 | Execute IPC request
    //bit 3 = Y2 | Acknowledge IPC request
    //bit 4 = IX1 | Execute ipc request send IPC interrupt
    //bit 5 = IX2 | Relaunch IPC sends IPC interrupt
    pub const HW_IPC_ARMCTRL: usize = 0xCD00_000Cusize; //from_exposed_addr_mut(0xCD00_000C);

    /// IPC Message Address (for BOTH ARM AND PPC)
    #[repr(transparent)]
    pub struct IpcMessageAddress(u32);

    impl IpcMessageAddress {
        #[must_use]
        pub const fn new() -> Self {
            Self(0)
        }

        #[must_use]
        pub fn read_ppc() -> Self {
            let hw_ipc_ppcmsg = from_exposed_addr_mut::<u32>(HW_IPC_PPCMSG);
            Self(unsafe { hw_ipc_ppcmsg.read_volatile() })
        }

        pub fn write_ppc(self) {
            let hw_ipc_ppcmsg = from_exposed_addr_mut::<u32>(HW_IPC_PPCMSG);
            unsafe { hw_ipc_ppcmsg.write_volatile(self.0) }
        }

        #[must_use]
        pub fn read_arm() -> Self {
            let hw_ipc_armmsg = from_exposed_addr_mut::<u32>(HW_IPC_ARMMSG);
            Self(unsafe { hw_ipc_armmsg.read_volatile() })
        }

        pub fn write_arm(self) {
            let hw_ipc_armmsg = from_exposed_addr_mut::<u32>(HW_IPC_ARMMSG);
            unsafe { hw_ipc_armmsg.write_volatile(self.0) }
        }

        pub const fn address(&self) -> u32 {
            self.0
        }

        /// # Panics:
        /// This function will panic if `address` is not in the MEM2 physical address space
        /// (`0x1000_0000` - `0x13FF_FFFF`)
        #[must_use]
        pub const fn with_address(mut self, address: u32) -> Self {
            /*assert!(
                (0x1000_0000..0x1400_0000).contains(&address),
                "Address must be in physical space"
            );*/

            self.0 = bitfrob::u32_with_value(0, 31, self.0, address);
            self
        }
    }

    /// `PowerPC` IPC Control
    #[repr(transparent)]
    pub struct PpcIpcControl(pub u32);

    impl PpcIpcControl {
        #[must_use]
        pub const fn new() -> Self {
            Self(0)
        }

        #[must_use]
        pub fn read() -> Self {
            let hw_ipc_ppcctrl = from_exposed_addr_mut::<u32>(HW_IPC_PPCCTRL);
            Self(unsafe { hw_ipc_ppcctrl.read_volatile() })
        }

        pub fn write(self) {
            let hw_ipc_ppcctrl = from_exposed_addr_mut::<u32>(HW_IPC_PPCCTRL);
            unsafe { hw_ipc_ppcctrl.write_volatile(self.0) }
        }

        pub const fn execute(&self) -> bool {
            bitfrob::u32_get_bit(0, self.0)
        }

        #[must_use]
        pub const fn with_execute(mut self, execute: bool) -> Self {
            self.0 = bitfrob::u32_with_bit(0, self.0, execute);
            self
        }

        pub const fn acknowledge(&self) -> bool {
            bitfrob::u32_get_bit(1, self.0)
        }

        #[must_use]
        pub const fn with_acknowledge(mut self, acknowledge: bool) -> Self {
            self.0 = bitfrob::u32_with_bit(1, self.0, acknowledge);
            self
        }

        pub const fn reply(&self) -> bool {
            bitfrob::u32_get_bit(2, self.0)
        }

        #[must_use]
        pub const fn with_reply(mut self, reply: bool) -> Self {
            self.0 = bitfrob::u32_with_bit(2, self.0, reply);
            self
        }

        pub const fn relaunch(&self) -> bool {
            bitfrob::u32_get_bit(3, self.0)
        }

        #[must_use]
        pub const fn with_relaunch(mut self, relaunch: bool) -> Self {
            self.0 = bitfrob::u32_with_bit(3, self.0, relaunch);
            self
        }

        pub const fn reply_interrupt(&self) -> bool {
            bitfrob::u32_get_bit(4, self.0)
        }

        #[must_use]
        pub const fn with_reply_interrupt(mut self, reply_interrupt: bool) -> Self {
            self.0 = bitfrob::u32_with_bit(4, self.0, reply_interrupt);
            self
        }

        pub const fn acknowledge_interrupt(&self) -> bool {
            bitfrob::u32_get_bit(5, self.0)
        }

        #[must_use]
        pub const fn with_acknowledge_interrupt(mut self, acknowledge_interrupt: bool) -> Self {
            self.0 = bitfrob::u32_with_bit(5, self.0, acknowledge_interrupt);
            self
        }
    }

    /// ARM IPC Control
    #[repr(transparent)]
    pub struct ArmIpcControl(u32);

    impl ArmIpcControl {
        #[must_use]
        pub const fn new() -> Self {
            Self(0)
        }

        #[must_use]
        pub fn read() -> Self {
            let hw_ipc_armctrl = from_exposed_addr_mut::<u32>(HW_IPC_ARMCTRL);
            Self(unsafe { hw_ipc_armctrl.read_volatile() })
        }

        pub fn write(self) {
            let hw_ipc_armctrl = from_exposed_addr_mut::<u32>(HW_IPC_ARMCTRL);
            unsafe { hw_ipc_armctrl.write_volatile(self.0) }
        }

        pub const fn execute(&self) -> bool {
            bitfrob::u32_get_bit(2, self.0)
        }

        #[must_use]
        pub const fn with_execute(mut self, execute: bool) -> Self {
            self.0 = bitfrob::u32_with_bit(2, self.0, execute);
            self
        }

        pub const fn acknowledge(&self) -> bool {
            bitfrob::u32_get_bit(3, self.0)
        }

        #[must_use]
        pub const fn with_acknowledge(mut self, acknowledge: bool) -> Self {
            self.0 = bitfrob::u32_with_bit(3, self.0, acknowledge);
            self
        }

        pub const fn reply(&self) -> bool {
            bitfrob::u32_get_bit(0, self.0)
        }

        #[must_use]
        pub const fn with_reply(mut self, reply: bool) -> Self {
            self.0 = bitfrob::u32_with_bit(0, self.0, reply);
            self
        }

        pub const fn relaunch(&self) -> bool {
            bitfrob::u32_get_bit(1, self.0)
        }

        #[must_use]
        pub const fn with_relaunch(mut self, relaunch: bool) -> Self {
            self.0 = bitfrob::u32_with_bit(1, self.0, relaunch);
            self
        }

        pub const fn execute_interrupt(&self) -> bool {
            bitfrob::u32_get_bit(4, self.0)
        }

        #[must_use]
        pub const fn with_execute_interrupt(mut self, execute_interrupt: bool) -> Self {
            self.0 = bitfrob::u32_with_bit(4, self.0, execute_interrupt);
            self
        }

        pub const fn relaunch_interrupt(&self) -> bool {
            bitfrob::u32_get_bit(5, self.0)
        }

        #[must_use]
        pub const fn with_relaunch_interrupt(mut self, relaunch_interrupt: bool) -> Self {
            self.0 = bitfrob::u32_with_bit(5, self.0, relaunch_interrupt);
            self
        }
    }

    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum IpcCommand {
        Open = 1,
        Close = 2,
        Read = 3,
        Write = 4,
        Seek = 5,
        Ioctl = 6,
        Ioctlv = 7,
        Reply = 8,
        Interrupt = 9,
    }

    #[repr(i32)]
    pub enum IpcAccessMode {
        Read = 1,
        Write = 2,
        ReadWrite = 3,
    }

    impl From<IpcAccessMode> for u32 {
        fn from(value: IpcAccessMode) -> Self {
            match value {
                IpcAccessMode::Read => 1,
                IpcAccessMode::Write => 2,
                IpcAccessMode::ReadWrite => 3,
            }
        }
    }

    impl From<IpcAccessMode> for usize {
        fn from(value: IpcAccessMode) -> Self {
            match value {
                IpcAccessMode::Read => 1,
                IpcAccessMode::Write => 2,
                IpcAccessMode::ReadWrite => 3,
            }
        }
    }

    impl TryFrom<usize> for IpcAccessMode {
        type Error = ();
        fn try_from(value: usize) -> Result<Self, Self::Error> {
            match value {
                1 => Ok(Self::Read),
                2 => Ok(Self::Write),
                3 => Ok(Self::ReadWrite),
                _ => Err(()),
            }
        }
    }

    #[repr(u32)]
    pub enum IpcSeekMode {
        Start = 0,
        Current = 1,
        End = 2,
    }

    impl From<IpcSeekMode> for u32 {
        fn from(value: IpcSeekMode) -> Self {
            match value {
                IpcSeekMode::Start => 0,
                IpcSeekMode::Current => 1,
                IpcSeekMode::End => 2,
            }
        }
    }

    #[repr(C)]
    #[derive(Debug)]
    pub enum IpcError {
        PermissionDenied(IpcRequest),
        FileExists(IpcRequest),
        InvalidArg(IpcRequest),
        FileNotFound(IpcRequest),
        ResourceBusy(IpcRequest),
        Ecc(IpcRequest),
        AllocFailed(IpcRequest),
        CorruptedFile(IpcRequest),
        TooManyFiles(IpcRequest),
        PathToLong(IpcRequest),
        FileisAlreadyOpen(IpcRequest),
        DirectoryNotEmpty(IpcRequest),
        Fatal(IpcRequest),
        Other(&'static str),
    }

    //Args setup found here: https://wiibrew.org/wiki/IOS
    //Struct is found here: https://wiibrew.org/wiki/IOS/Resource_request

    //TODO: impl drop to avoid leaking data out of boxes and CString

    #[repr(C, align(32))]
    pub struct IpcRequest {
        pub cmd: IpcCommand,
        pub ret: i32,
        pub file_desciptor: u32,
        pub args: [u32; 5],
        pub callback: Option<Box<dyn Fn(*mut IpcRequest)>>,
    }

    impl Default for IpcRequest {
        fn default() -> Self {
            Self::new()
        }
    }

    impl IpcRequest {
        #[must_use]
        pub fn new() -> Self {
            Self {
                cmd: IpcCommand::Open,
                ret: 0,
                file_desciptor: 0,
                args: [0u32; 5],
                callback: None,
            }
        }

        #[must_use]
        pub fn open(file_path: CString, access_mode: IpcAccessMode) -> Self {
            Self {
                cmd: IpcCommand::Open,
                ret: 0,
                file_desciptor: 0,
                args: [
                    file_path
                        .into_raw()
                        .map_addr(|addr| addr & 0x1FFF_FFFF)
                        .expose_addr()
                        .try_into()
                        .unwrap(),
                    access_mode.into(),
                    0,
                    0,
                    0,
                ],
                callback: None,
            }
        }

        #[must_use]
        pub fn close(file_desciptor: u32) -> Self {
            Self {
                cmd: IpcCommand::Close,
                ret: 0,
                file_desciptor,
                args: [0u32; 5],
                callback: None,
            }
        }

        //TODO: Make sure read fills entire capacity
        //Read must set_len = capacity at end of call
        #[must_use]
        pub fn read(file_desciptor: u32, read_buf: Vec<u8>) -> Self {
            let (ptr, length, capacity) = read_buf.into_raw_parts();
            Self {
                cmd: IpcCommand::Read,
                ret: 0,
                file_desciptor,
                args: [
                    ptr.map_addr(|addr| addr & 0x1FFF_FFFF)
                        .expose_addr()
                        .try_into()
                        .unwrap(),
                    capacity.try_into().unwrap(),
                    length.try_into().unwrap(),
                    0,
                    0,
                ],
                callback: None,
            }
        }

        #[must_use]
        pub fn write(file_desciptor: u32, write_buf: Vec<u8>) -> Self {
            let (ptr, length, capacity) = write_buf.into_raw_parts();
            Self {
                cmd: IpcCommand::Write,
                ret: 0,
                file_desciptor,
                args: [
                    ptr.map_addr(|addr| addr & 0x1FFF_FFFF)
                        .expose_addr()
                        .try_into()
                        .unwrap(),
                    length.try_into().unwrap(),
                    capacity.try_into().unwrap(),
                    0,
                    0,
                ],
                callback: None,
            }
        }

        #[must_use]
        pub fn seek(file_desciptor: u32, offset: u32, seek_mode: IpcSeekMode) -> Self {
            Self {
                cmd: IpcCommand::Seek,
                ret: 0,
                file_desciptor,
                args: [offset, seek_mode.into(), 0, 0, 0],
                callback: None,
            }
        }

        #[must_use]
        pub fn ioctl<I, O>(file_desciptor: u32, ioctl: u32, input: Box<I>, output: Box<O>) -> Self {
            Self {
                cmd: IpcCommand::Ioctl,
                ret: 0,
                file_desciptor,
                args: [
                    ioctl,
                    Box::into_raw(input)
                        .map_addr(|addr| addr & 0x1FFF_FFFF)
                        .expose_addr()
                        .try_into()
                        .unwrap(),
                    core::mem::size_of::<I>().try_into().unwrap(),
                    Box::into_raw(output)
                        .map_addr(|addr| addr & 0x1FFF_FFFF)
                        .expose_addr()
                        .try_into()
                        .unwrap(),
                    core::mem::size_of::<O>().try_into().unwrap(),
                ],
                callback: None,
            }
        }

        // The IoVec slice **MUST** be the size of COUNT_IN + COUNT_OUT
        #[must_use]
        pub fn ioctlv<const COUNT_IN: u32, const COUNT_OUT: u32>(
            file_desciptor: u32,
            ioctl: u32,
            iovecs: Box<[IoVec]>,
        ) -> Self {
            Self {
                cmd: IpcCommand::Ioctlv,
                ret: 0,
                file_desciptor,
                args: [
                    ioctl,
                    COUNT_IN,
                    COUNT_OUT,
                    Box::into_raw(iovecs)
                        .map_addr(|addr| addr & 0x1FFF_FFFF)
                        .expose_addr()
                        .try_into()
                        .unwrap(),
                    0,
                ],
                callback: None,
            }
        }

        #[must_use]
        pub fn with_callback<F>(mut self, callback: F) -> Self
        where
            //TODO: THIS IS PROBABLY WRONG, it might not live long enough ig
            F: Fn(*mut Self) + 'static,
        {
            self.callback = Some(Box::new(callback));
            self
        }

        /// # Errors
        /// see `IpcError`
        #[inline(never)]
        pub fn send(self) -> Result<Self, IpcError> {
            let has_callback = self.callback.is_some();
            let cmd = self.cmd;
            let mut ptr = Box::into_raw(Box::new(self));

            //Flush request data
            dc_flush_range(
                ptr.cast::<u8>(),
                core::mem::size_of::<Self>().next_multiple_of(32),
            );

            //Move pointer to physical address space
            ptr = ptr.map_addr(|addr| addr & 0x1FFF_FFFF);

            //Put pointer into PowerPC IpcMessageAddress
            IpcMessageAddress::new()
                .with_address(ptr.expose_addr().try_into().unwrap())
                .write_ppc();

            //On first boot ipc sends ack to know its alive
            if PpcIpcControl::read().acknowledge() {
                PpcIpcControl::new()
                    .with_acknowledge(true)
                    .with_execute(true)
                    .write();
            } else if has_callback {
                PpcIpcControl::new()
                    .with_execute(true)
                    .with_acknowledge_interrupt(true)
                    .with_reply_interrupt(true)
                    .write();
                return Ok(unsafe { ptr.map_addr(|addr| addr | 0x8000_0000).read_unaligned() });
            } else {
                PpcIpcControl::new()
                    .with_execute(true)
                    .with_reply_interrupt(false)
                    .with_acknowledge_interrupt(false)
                    .write();
            }

            //Wait for acknowledge
            while !PpcIpcControl::read().acknowledge() {
                core::hint::spin_loop();
            }

            //Send that we got the acknowledge
            PpcIpcControl::new().with_acknowledge(true).write();

            //Wait for a reply
            while !PpcIpcControl::read().reply() {
                core::hint::spin_loop();
            }

            //We got a reply build an ipc request
            let ipc_message_address = IpcMessageAddress::read_arm().address();
            let mut ipc_message_ptr =
                from_exposed_addr_mut::<Self>(ipc_message_address.try_into().unwrap());

            //Map back to virtual
            ipc_message_ptr = ipc_message_ptr.map_addr(|addr| addr | 0x8000_0000);

            //Acknowledge the reply
            PpcIpcControl::new().with_reply(true).write();

            unsafe {
                let mut ipc = Box::from_raw(ipc_message_ptr);
                ipc.cmd = cmd;

                match ipc.ret {
                    -1 | -102 => Err(IpcError::PermissionDenied(*ipc)),
                    -2 | -105 => Err(IpcError::FileExists(*ipc)),
                    -4 | -101 => Err(IpcError::InvalidArg(*ipc)),
                    -6 | -106 => Err(IpcError::FileNotFound(*ipc)),
                    -8 | -118 => Err(IpcError::ResourceBusy(*ipc)),
                    -12 | -114 => Err(IpcError::Ecc(*ipc)),
                    -103 => Err(IpcError::CorruptedFile(*ipc)),
                    -107 | -109 => Err(IpcError::TooManyFiles(*ipc)),
                    -22 | -108 => Err(IpcError::AllocFailed(*ipc)),
                    -110 | -116 => Err(IpcError::PathToLong(*ipc)),
                    -111 => Err(IpcError::FileisAlreadyOpen(*ipc)),
                    -115 => Err(IpcError::DirectoryNotEmpty(*ipc)),
                    -119 => Err(IpcError::Fatal(*ipc)),
                    _ => Ok(*ipc),
                }
            }
        }

        pub fn take_output<O>(&mut self) -> Option<Box<O>> {
            if self.cmd != IpcCommand::Ioctl
                || self.args[3] == 0
                || self.args[4] == 0
                || core::mem::size_of::<O>() != self.args[4].try_into().unwrap()
            {
                return None;
            }

            let ret = Some(unsafe {
                Box::from_raw(from_exposed_addr_mut::<O>(
                    (self.args[3] | 0x8000_0000).try_into().unwrap(),
                ))
            });

            self.args[3] = 0;
            self.args[4] = 0;

            ret
        }

        pub fn take_buf(&mut self) -> Option<Vec<u8>> {
            if self.cmd != IpcCommand::Read || self.args[0] == 0 || self.args[1] == 0 {
                return None;
            }

            let ret = Some(unsafe {
                let mut vec = Vec::from_raw_parts(
                    from_exposed_addr_mut((self.args[0] | 0x8000_0000).try_into().unwrap()),
                    0,
                    self.args[1].try_into().unwrap(),
                );
                vec.set_len(vec.capacity());
                vec
            });

            self.args[1] = 0;
            self.args[2] = 0;

            ret
        }
    }

    impl Debug for IpcRequest {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_fmt(format_args!(
                "IpcRequest: [ cmd: {:?}, ret: {}, fd: {}, args: {:?} ]",
                self.cmd, self.ret, self.file_desciptor, self.args,
            ))
        }
    }
}
