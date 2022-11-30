use bit_field::BitField;

pub struct Instant {
    pub ticks: u64,
}

impl Instant {
    pub fn now() -> Self {
        let mut time1 = 0u32;
        let time2: u32;
        let mut _time3 = 0u32;

        let mut instant = 0u64;
        unsafe {
            core::arch::asm!(
                "1: mftbu {time1}",
                "mftb {time2}",
                "mftbu {time3}",
                "cmpw {time1},{time3}",
                "bne 1b",
                time1 = inout(reg) time1,
                time2 = out(reg) time2,
                 time3 = inout(reg) _time3,
            );
        }

        instant.set_bits(0..=31, time2.into());
        instant.set_bits(32..=63, time1.into());

        Self { ticks: instant }
    }

    pub fn secs(&self) -> u64 {
        self.ticks / (TB_TIMER_CLOCK * 1000)
    }

    pub fn millisecs(&self) -> u64 {
        self.ticks / TB_TIMER_CLOCK
    }

    pub fn microsecs(&self) -> u64 {
        (self.ticks * 8) / (TB_TIMER_CLOCK / 125)
    }

    pub fn nanosecs(&self) -> u64 {
        (self.ticks * 8000) / (TB_TIMER_CLOCK / 125)
    }
}

pub fn set_time(time: u64) {
    let time_upper: u32 = time.get_bits(32..=63).try_into().unwrap();
    let time_lower: u32 = time.get_bits(0..=31).try_into().unwrap();
    unsafe {
        core::arch::asm!(
            "mttbu {time_upper}",
            "mttbl {time_lower}",
            time_upper = in(reg) time_upper,
            time_lower = in(reg) time_lower,
        );
    }
}

pub const TB_CORE_CLOCK: u64 = 729000000;
pub const TB_BUS_CLOCK: u64 = 243000000;
pub const TB_TIMER_CLOCK: u64 = TB_BUS_CLOCK / 4000;
