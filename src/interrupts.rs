use core::arch::asm;

pub fn disable() -> usize {
    let mut cookie = 0usize;
    unsafe {
        asm!("mfmsr 3",
          "rlwinm. 3,{0},0,17,15",
          "mtmsr 3",
          "extrwi. {0},{0},1,16",
          inout(reg) cookie,
        );
    }
    cookie
}
