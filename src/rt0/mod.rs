use crate::os::LinkerSymbol;

core::arch::global_asm! {
    include_str!("init.S"),
    options(raw)
}

core::arch::global_asm! {
    include_str!("hw_init.S"),
    options(raw)
}

core::arch::global_asm! {
    include_str!("cache.S"),
    options(raw)
}

extern "C" {
    pub fn data_cache_flush_range_no_sync(data: *const u8, len: u32);
    pub fn instruction_cache_invalidate_range(data: *const u8, len: u32);
}


core::arch::global_asm! {
    include_str!("exception_handler.S"),
    options(raw)
}

extern "C" {
    pub static EXCEPTION_HANDLER_START: LinkerSymbol;
    pub static EXCEPTION_HANDLER_END: LinkerSymbol;
    pub static SYSTEMCALL_HANDLER_START: LinkerSymbol;
    pub static SYSTEMCALL_HANDLER_END: LinkerSymbol;
}


extern "C" {
    pub fn data_cache_flush_range(addr: *mut u8, len: u32);
}
