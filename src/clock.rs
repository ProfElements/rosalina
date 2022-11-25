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
        self.ticks / ((243000000 / 4000) * 1000)
    }

    pub fn millisecs(&self) -> u64 {
        self.ticks / (243000000 / 4000)
    }

    pub fn microsecs(&self) -> u64 {
        (self.ticks * 8) / ((243000000 / 4000) / 125)
    }

    pub fn nanosecs(&self) -> u64 {
        (self.ticks * 8000) / ((243000000 / 4000) / 125)
    }
}
