use bit_field::BitField;

use crate::mmio::vi::Enabled;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct MachineStateRegister(u32);

pub enum PowerManagement {
    Normal,
    Reduced,
}

impl From<bool> for PowerManagement {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Reduced,
            false => Self::Normal,
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

pub enum Endianness {
    Little,
    Big,
}

impl From<bool> for Endianness {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Little,
            false => Self::Big,
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

pub enum Priviledge {
    Supervisor,
    User,
}

impl From<bool> for Priviledge {
    fn from(value: bool) -> Self {
        match value {
            true => Priviledge::Supervisor,
            false => Priviledge::User,
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

pub enum FloatingPointExceptionMode {
    Disabled,
    Nonrecoverable,
    Recoverable,
    Precise,
}

pub enum ExceptionPrefix {
    Zero,
    Ffff,
}

impl From<bool> for ExceptionPrefix {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Ffff,
            false => Self::Zero,
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

pub enum Marked {
    Nonmarked,
    Marked,
}

impl From<bool> for Marked {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Marked,
            false => Self::Nonmarked,
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

pub enum ExceptionMode {
    Nonrecoverable,
    Recoverable,
}

impl From<bool> for ExceptionMode {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Recoverable,
            false => Self::Nonrecoverable,
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
            )
        }
        Self(reg)
    }

    pub fn write(self) {
        let reg = self.0;
        unsafe {
            core::arch::asm!(
                "mtmsr {reg}",
                reg = in(reg) reg,
            )
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
