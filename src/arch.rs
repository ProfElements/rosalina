use bit_field::BitField;

use crate::mmio::vi::Enabled;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct MachineStateRegister(u32);

#[derive(Copy, Clone)]
pub enum PowerManagement {
    Normal,
    Reduced,
}

impl From<bool> for PowerManagement {
    fn from(value: bool) -> Self {
        if value {
            Self::Reduced
        } else {
            Self::Normal
        }
    }
}

impl From<PowerManagement> for bool {
    fn from(value: PowerManagement) -> Self {
        match value {
            PowerManagement::Reduced => true,
            PowerManagement::Normal => false,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Endianness {
    Little,
    Big,
}

impl From<bool> for Endianness {
    fn from(value: bool) -> Self {
        if value {
            Self::Little
        } else {
            Self::Big
        }
    }
}

impl From<Endianness> for bool {
    fn from(value: Endianness) -> Self {
        match value {
            Endianness::Little => true,
            Endianness::Big => false,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Priviledge {
    Supervisor,
    User,
}

impl From<bool> for Priviledge {
    fn from(value: bool) -> Self {
        if value {
            Self::Supervisor
        } else {
            Self::User
        }
    }
}

impl From<Priviledge> for bool {
    fn from(value: Priviledge) -> Self {
        match value {
            Priviledge::Supervisor => true,
            Priviledge::User => false,
        }
    }
}

#[derive(Copy, Clone)]
pub enum FloatingPointExceptionMode {
    Disabled,
    Nonrecoverable,
    Recoverable,
    Precise,
}

#[derive(Copy, Clone)]
pub enum ExceptionPrefix {
    Zero,
    Ffff,
}

impl From<bool> for ExceptionPrefix {
    fn from(value: bool) -> Self {
        if value {
            Self::Ffff
        } else {
            Self::Zero
        }
    }
}

impl From<ExceptionPrefix> for bool {
    fn from(value: ExceptionPrefix) -> Self {
        match value {
            ExceptionPrefix::Ffff => true,
            ExceptionPrefix::Zero => false,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Marked {
    Nonmarked,
    Marked,
}

impl From<bool> for Marked {
    fn from(value: bool) -> Self {
        if value {
            Self::Marked
        } else {
            Self::Nonmarked
        }
    }
}

impl From<Marked> for bool {
    fn from(value: Marked) -> Self {
        match value {
            Marked::Marked => true,
            Marked::Nonmarked => false,
        }
    }
}

#[derive(Copy, Clone)]
pub enum ExceptionMode {
    Nonrecoverable,
    Recoverable,
}

impl From<bool> for ExceptionMode {
    fn from(value: bool) -> Self {
        if value {
            Self::Recoverable
        } else {
            Self::Nonrecoverable
        }
    }
}

impl From<ExceptionMode> for bool {
    fn from(value: ExceptionMode) -> Self {
        match value {
            ExceptionMode::Recoverable => true,
            ExceptionMode::Nonrecoverable => false,
        }
    }
}

impl MachineStateRegister {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn read() -> Self {
        let reg;
        unsafe {
            core::arch::asm!(
                "mfmsr {reg}",
                reg = out(reg) reg,
            );
        }
        Self(reg)
    }

    pub fn write(self) {
        let reg = self.0;
        unsafe {
            core::arch::asm!(
                "mtmsr {reg}",
                reg = in(reg) reg,
            );
        }
    }

    pub fn power_management(&self) -> PowerManagement {
        self.0.get_bit(14).into()
    }

    pub fn with_power_management(&mut self, power: PowerManagement) -> &mut Self {
        self.0.set_bit(14, power.into());
        self
    }

    pub fn exception_little_endian_mode(&self) -> Endianness {
        self.0.get_bit(16).into()
    }

    pub fn with_exception_little_endian_mode(&mut self, endian: Endianness) -> &mut Self {
        self.0.set_bit(16, endian.into());
        self
    }

    pub fn external_interrupt_enabled(&self) -> Enabled {
        self.0.get_bit(15).into()
    }

    pub fn with_external_interrupt_enabled(&mut self, enable: Enabled) -> &mut Self {
        self.0.set_bit(15, enable.into());
        self
    }

    pub fn priviledge(&self) -> Priviledge {
        self.0.get_bit(14).into()
    }

    pub fn with_priviledge(&mut self, priviledge: Priviledge) -> &mut Self {
        self.0.set_bit(14, priviledge.into());
        self
    }

    pub fn floating_point_enable(&self) -> Enabled {
        self.0.get_bit(13).into()
    }

    pub fn with_floating_point_enable(&mut self, enable: Enabled) -> &mut Self {
        self.0.set_bit(13, enable.into());
        self
    }

    pub fn machine_check_enable(&self) -> Enabled {
        self.0.get_bit(12).into()
    }

    pub fn with_machine_check_enable(&mut self, enable: Enabled) -> &mut Self {
        self.0.set_bit(12, enable.into());
        self
    }

    pub fn step_trace_enable(&self) -> Enabled {
        self.0.get_bit(10).into()
    }

    pub fn with_step_trace_enable(&mut self, enable: Enabled) -> &mut Self {
        self.0.set_bit(10, enable.into());
        self
    }

    pub fn branch_trace_enable(&self) -> Enabled {
        self.0.get_bit(9).into()
    }

    pub fn with_branch_trace_enable(&mut self, enable: Enabled) -> &mut Self {
        self.0.set_bit(9, enable.into());
        self
    }

    pub fn floating_point_exception_mode(&self) -> FloatingPointExceptionMode {
        match (self.0.get_bit(11), self.0.get_bit(8)) {
            (false, false) => FloatingPointExceptionMode::Disabled,
            (false, true) => FloatingPointExceptionMode::Nonrecoverable,
            (true, false) => FloatingPointExceptionMode::Recoverable,
            (true, true) => FloatingPointExceptionMode::Precise,
        }
    }

    pub fn with_floating_point_exception_mode(
        &mut self,
        mode: FloatingPointExceptionMode,
    ) -> &mut Self {
        match mode {
            FloatingPointExceptionMode::Disabled => {
                self.0.set_bit(11, false);
                self.0.set_bit(8, false);
            }
            FloatingPointExceptionMode::Nonrecoverable => {
                self.0.set_bit(11, false);
                self.0.set_bit(8, true);
            }
            FloatingPointExceptionMode::Recoverable => {
                self.0.set_bit(11, true);
                self.0.set_bit(8, false);
            }
            FloatingPointExceptionMode::Precise => {
                self.0.set_bit(11, true);
                self.0.set_bit(8, true);
            }
        }
        self
    }

    pub fn exception_prefix(&self) -> ExceptionPrefix {
        self.0.get_bit(6).into()
    }

    pub fn with_exception_prefix(&mut self, prefix: ExceptionPrefix) -> &mut Self {
        self.0.set_bit(6, prefix.into());
        self
    }

    pub fn instruction_translation_enable(&self) -> Enabled {
        self.0.get_bit(5).into()
    }

    pub fn with_instruction_translation_enable(&mut self, enable: Enabled) -> &mut Self {
        self.0.set_bit(5, enable.into());
        self
    }

    pub fn data_translation_enable(&self) -> Enabled {
        self.0.get_bit(4).into()
    }

    pub fn with_data_translation_enable(&mut self, enable: Enabled) -> &mut Self {
        self.0.set_bit(4, enable.into());
        self
    }

    pub fn performance_marked_mode(&self) -> Marked {
        self.0.get_bit(2).into()
    }

    pub fn with_performance_marked_mode(&mut self, marked: Marked) -> &mut Self {
        self.0.set_bit(2, marked.into());
        self
    }

    pub fn external_exception_mode(&self) -> ExceptionMode {
        self.0.get_bit(1).into()
    }

    pub fn with_external_exception_mode(&mut self, mode: ExceptionMode) -> &mut Self {
        self.0.set_bit(1, mode.into());
        self
    }

    pub fn little_endian_mode(&self) -> Endianness {
        self.0.get_bit(0).into()
    }

    pub fn with_little_endian_mode(&mut self, endian: Endianness) -> &mut Self {
        self.0.set_bit(0, endian.into());
        self
    }
}

bitflags::bitflags! {
    pub struct HID0Flags: u32 {
        const NOOPTI = 1 << 0;
        ///Branch History Table Enable
        const BHT = 1 << 2;
        const ABE = 1 << 3;
        /// Branch Target Instruction Cache Enable
        const BTIC = 1 << 5;
        /// Data Cache Flush Assist Enable
        const DCFA = 1 << 6;
        const SGE = 1 << 7;
        const IFEM = 1 << 8;
        const SPD = 1 << 9;
        /// Data Cache Flash Invalidate
        const DCFI = 1 << 10;
        /// Instruction Cache Flash Invalidate
        const ICFI = 1 << 11;
        /// Data Cache Lock
        const DLOCK = 1 << 12;
        /// Instruction Cache Lock
        const ILOCK = 1 << 13;
        /// Data Cache Enable
        const DCE = 1 << 14;
        /// Instruction Cache Enable
        const ICE = 1 << 15;
        /// No Hard Reset
        const NHR = 1 << 16;
        ///Dynamic Power Management
        const DPM = 1 << 20;
        const SLEEP = 1 << 21;
        const NAP = 1 << 22;
        const DOZE = 1 << 23;
        const PAR = 1 << 24;
        const ECLK = 1 << 25;
        const BCLK = 1 << 27;
        const EBD = 1 << 28;
        const EBA = 1 << 29;
        const DBP = 1 << 30;
        const EMCP = 1 << 31;
    }
}

bitflags::bitflags! {
    pub struct HID2Flags: u32 {
        /// Write Gather Pipe Enable
        const WPE = 1 << 30;
        /// Paired Single Enable
        const PSE = 1 << 29;
        /// Locked Cache Enable
        const LCE = 1 << 28;
        //DMAQL (DMA Queue Length) 27:24
        /// DMA Cache Hit Error
        const DCHERR = 1 << 23;
        /// DMA Normal Cache Error
        const DNCERR = 1 << 22;
        /// DMA Cache Miss Error
        const DMCERR = 1 << 21;
        /// DMA Queue Overflow Error
        const DQOERR = 1 << 20;
        // DMA Cache Hit Error Enable
        const DCHEE = 1 << 19;
        /// DMA Cache Miss Error Enable
        const DCMEE = 1 << 18;
        /// DMA Queue Overflow Error Enable
        const DQOEE = 1 << 17;
    }
}

bitflags::bitflags! {
    pub struct HID4Flags: u32 {
        /// HID4 Access
        const H4A = 1 << 31;
        //L2FM (L2 Fetch Mode) 30:29
        //BPD (Bus Pipeline Depth) 28:27
        /// L2 Second Castout Buffer Enable
        const BCO = 1 << 26;
        /// Secondary Bat Enable
        const SBE = 1 << 25;
        //Paired Single 1 Control
        const PS1_CTL = 1 << 24;
        /// Data Bus Parking
        const DBP = 1 << 22;
        /// L2 Miss under Miss Enable
        const L2MUM = 1 << 21;
        //L2 Complete Castout Flash Invalidate
        const L2_CCFI = 1 << 20;
        //Paired Single 2 Control
        const PS2_CTL = 1 << 19;
    }
}

bitflags::bitflags! {
    pub struct UBATFlags: u32 {
        const VP = 1 << 0;
        const VS = 1 << 1;
        // BL (Block Length) 2:12
        // BEPI (Block Effective Page Index) 31:17
    }
}

impl UBATFlags {
    pub const fn with_block_effective_page_index(self, bepi: u32) -> Self {
        Self::from_bits_retain(bitfrob::u32_with_value(16, 31, self.bits(), bepi))
    }

    pub const fn with_block_length(self, bl: BlockLength) -> Self {
        Self::from_bits_retain(bitfrob::u32_with_value(2, 12, self.bits(), bl as u32))
    }
}

#[repr(u32)]
pub enum BlockLength {
    KBytes128 = 0,
    Kbytes256 = 0b000_0000_0001,
    KBytes512 = 0b000_0000_0011,
    MBytes1 = 0b000_0000_0111,
    MBytes2 = 0b000_0000_1111,
    MBytes4 = 0b000_0001_1111,
    MBytes8 = 0b000_0011_1111,
    MBytes16 = 0b000_0111_1111,
    MBytes32 = 0b000_1111_1111,
    MBytes64 = 0b001_1111_1111,
    MBytes128 = 0b011_1111_1111,
    MBytes256 = 0b111_1111_1111,
}

impl From<BlockLength> for u32 {
    fn from(value: BlockLength) -> Self {
        value as u32
    }
}

bitflags::bitflags! {
    pub struct LBATFlags: u32 {
        // PP (Memory Protection Parts) 0:1
        const G = 1 << 3;
        const M = 1 << 4;
        const I = 1 << 5;
        const W = 1 << 6;
        // BRPN (Block Physical Retain Numbers) 31:16
    }
}

impl LBATFlags {
    pub const fn with_memory_protection_parts(self, pp: MemoryProtectionParts) -> Self {
        Self::from_bits_retain(bitfrob::u32_with_value(0, 1, self.bits(), pp as u32))
    }

    pub const fn with_block_physical_number(self, brpn: u32) -> Self {
        Self::from_bits_retain(bitfrob::u32_with_value(16, 31, self.bits(), brpn))
    }
}

#[repr(u32)]
pub enum MemoryProtectionParts {
    None = 0b00,
    Read = 0b01,
    ReadWrite = 0b10,
}

impl From<MemoryProtectionParts> for u32 {
    fn from(value: MemoryProtectionParts) -> Self {
        value as u32
    }
}

bitflags::bitflags! {
    pub struct MSRFlags: u32 {
        const LE = 1 << 0;
        const RI = 1 << 1;
        const DR = 1 << 4;
        const IR = 1 << 5;
        const IP = 1 << 6;
        const FE1 = 1 << 8;
        const BE = 1 << 9;
        const SE = 1 << 10;
        const FE0 = 1 << 11;
        const ME = 1 << 12;
        const FP = 1 << 13;
        const PR = 1 << 14;
        const EE = 1 << 15;
        const ILE = 1 << 16;
        const POW = 1 << 18;
    }
}
