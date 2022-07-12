
core::arch::global_asm!{
    include_str!("init.S"), 
    options(raw) 
}

core::arch::global_asm!{
    include_str!("hw_init.S"), 
    options(raw) 
}

core::arch::global_asm!{
    include_str!("cache.S"), 
    options(raw) 
}

extern "C" {
    pub fn data_cache_flush_range(addr: *mut u8, len: u32); 
}
