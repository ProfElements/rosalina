
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
