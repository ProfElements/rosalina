use core::{ops::Sub, time::Duration};

use bit_field::BitField;

#[derive(Copy, Clone)]
pub struct Instant {
    pub ticks: u64,
}

impl Instant {
    pub fn now() -> Self {
        let mut time1 = 0u32;
        let time2: u32;

        let mut instant = 0u64;
        unsafe {
            core::arch::asm!(
                "1: mftbu {time1}",
                "mftb {time2}",
                time1 = inout(reg) time1,
                time2 = out(reg) time2,
            );
        }

        instant.set_bits(0..32, time2.into());
        instant.set_bits(32..64, time1.into());

        Self { ticks: instant }
    }

    pub const fn secs(&self) -> u64 {
        self.ticks / (TB_TIMER_CLOCK * 1000)
    }

    pub const fn millisecs(&self) -> u64 {
        self.ticks / TB_TIMER_CLOCK
    }

    pub const fn microsecs(&self) -> u64 {
        (self.ticks * 8) / (TB_TIMER_CLOCK / 125)
    }

    pub const fn nanosecs(&self) -> u64 {
        (self.ticks * 8000) / (TB_TIMER_CLOCK / 125)
    }

    pub(crate) const fn from_ticks(ticks: u64) -> Self {
        Self { ticks }
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        Duration::from_nanos((*self - earlier).nanosecs())
    }

    pub fn elapsed(&self) -> Duration {
        Duration::from_nanos((Instant::now() - *self).nanosecs())
    }
}

impl Sub for Instant {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_ticks(self.ticks - rhs.ticks)
    }
}

impl Sub for &Instant {
    type Output = Instant;
    fn sub(self, rhs: Self) -> Self::Output {
        Instant::from_ticks(self.ticks - rhs.ticks)
    }
}

pub fn set_time(time: u64) {
    let time_upper: u32 = time.get_bits(32..64).try_into().unwrap();
    let time_lower: u32 = time.get_bits(0..32).try_into().unwrap();
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
