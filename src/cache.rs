#[no_mangle]
#[naked]
pub(crate) unsafe extern "C" fn __init_cache() -> ! {
    core::arch::asm!(
        "mflr 0",
        "stw 0,4(1)",
        "stwu 1,-16(1)",
        "stw 31,12(1)",
        "1:",
        "mfspr 3,{HID0}",
        "rlwinm. 0,3,0,16,16",
        "bne {ic_enabled}",
        "nop",
        "bl {ic_enable}",
        "b 1b",
        HID0 = const 1008,
        ic_enabled = sym ic_enabled,
        ic_enable = sym crate::cache::ic_enable,
        options(noreturn)
    )
}

#[naked]
pub extern "C" fn ic_flash_invalidate() -> ! {
    unsafe {
        core::arch::asm!(
            "mfspr 3,{HID0}",
            "ori 3,3,0x0800",
            "mtspr {HID0},3",
            "isync",
            "blr",
            HID0 = const 1008,
            options(noreturn)
        )
    }
}

#[naked]
pub extern "C" fn ic_enabled() -> ! {
    unsafe {
        core::arch::asm!(
            "1:",
            "mfspr 3,{HID0}",
            "rlwinm. 0,3,0,17,17",
            "bne {dc_enabled}",
            "nop",
            "bl {dc_enable}",
            "b 1b",
            HID0 = const 1008,
            dc_enabled = sym dc_enabled,
            dc_enable = sym dc_enable,
            options(noreturn)
        )
    }
}
#[naked]
pub extern "C" fn ic_enable() -> ! {
    unsafe {
        core::arch::asm!(
            "mfspr 3,{HID0}",
            "ori 3,3,0x8000",
            "mtspr {HID0},3",
            "isync",
            "blr",
            HID0 = const 1008,
            options(noreturn)
        )
    }
}

#[naked]
pub extern "C" fn ic_invalidate_range(ptr: *mut u8, len: usize) {
    unsafe {
        core::arch::asm!(
            "cmplwi 4,0",
            "blelr",
            "clrlwi. 5,3,27",
            "beq 1f",
            "addi 4,4,0x20",
            "1:",
            "addi 4,4,0x1F",
            "srwi 4,4,5",
            "mtctr 4",
            "2:",
            "icbi 0,3",
            "addi 3,3,0x20",
            "bdnz 2b",
            "sync",
            "isync",
            "blr",
            options(noreturn)
        )
    }
}

#[naked]
pub extern "C" fn dc_enabled() -> ! {
    unsafe {
        core::arch::asm!(
            "1:",
            "mfspr 3,{L2CR}",
            "clrrwi. 0,3,31",
            "bne {l2_enabled}",
            "nop",
            "bl {l2_init}",
            "bl {l2_enable}",
            "b 1b",
            L2CR = const 1017,
            l2_enabled = sym l2_enabled,
            l2_init = sym l2_init,
            l2_enable = sym l2_enable,
            options(noreturn)
        )
    }
}

#[naked]
pub extern "C" fn dc_enable() -> ! {
    unsafe {
        core::arch::asm!(
            "mfspr 3,{HID0}",
            "ori 3,3,0x4000",
            "mtspr {HID0},3",
            "isync",
            "blr",
            HID0 = const 1008,
            options(noreturn)
        )
    }
}

#[naked]
pub extern "C" fn dc_flush_range_no_sync(ptr: *mut u8, len: usize) {
    unsafe {
        core::arch::asm!(
            "cmplwi 4,0",
            "blelr",
            "clrlwi. 5,3,27",
            "beq 1f",
            "addi 4,4,0x20",
            "1:",
            "addi 4,4,0x1F",
            "srwi 4,4,5",
            "mtctr 4",
            "2:",
            "dcbf 0,3",
            "addi 3,3,0x20",
            "bdnz 2b",
            "blr",
            options(noreturn)
        )
    }
}

#[naked]
pub extern "C" fn dc_invalidate_range(ptr: *mut u8, len: usize) {
    unsafe {
        core::arch::asm!(
            "cmplwi 4,0",
            "blelr",
            "clrlwi. 5,3,27",
            "beq 1f",
            "addi 4,4,0x20",
            "1:",
            "addi 4,4,0x1f",
            "srwi 4,4,5",
            "mtctr 4",
            "2:",
            "dcbi 0,3",
            "addi 3,3,0x20",
            "bdnz 2b",
            "blr",
            options(noreturn)
        )
    }
}

#[naked]
pub extern "C" fn l2_enabled() -> ! {
    unsafe {
        core::arch::asm!(
            "lwz 0,20(1)",
            "lwz 31,12(1)",
            "addi 1,1,16",
            "mtlr 0",
            "blr",
            options(noreturn)
        )
    }
}

#[naked]
pub extern "C" fn l2_init() -> ! {
    unsafe {
        core::arch::asm!(
            "mflr 0",
            "stw 0,4(1)",
            "stwu 1,-16(1)",
            "stw 31,12(1)",
            "mfmsr 3",
            "mr 31,3",
            "sync",
            "li 3,48",
            "mtmsr 3",
            "sync",
            "bl {l2_disable}",
            "bl {l2_global_invalidate}",
            "mr 3,31",
            "mtmsr 3",
            "lwz 0,20(1)",
            "lwz 31,12(1)",
            "mtlr 0",
            "blr",
            l2_disable = sym l2_disable,
            l2_global_invalidate = sym l2_global_invalidate,
            options(noreturn)
        )
    }
}

#[naked]
pub extern "C" fn l2_enable() -> ! {
    unsafe {
        core::arch::asm!(
            "sync",
            "mfspr 3,{L2CR}",
            "oris 0,3,0x8000",
            "rlwinm. 3,0,0,11,9",
            "mtspr {L2CR},3",
            "sync",
            "blr",
            L2CR = const 1017,
            options(noreturn)
        )
    }
}

#[naked]
pub extern "C" fn l2_disable() -> ! {
    unsafe {
        core::arch::asm!(
            "sync",
            "mfspr 3,{L2CR}",
            "clrlwi 3,3,1",
            "mtspr {L2CR},3",
            "sync",
            "blr",
            L2CR = const 1017,
            options(noreturn)
        )
    }
}

#[naked]
pub extern "C" fn l2_global_invalidate() -> ! {
    unsafe {
        core::arch::asm!(
            "mflr 0",
            "stw 0,4(1)",
            "stwu 1,-8(1)",
            "bl {l2_disable}",
            "mfspr 3,{L2CR}",
            "ori 3,3,0x0020",
            "mtspr {L2CR},3",
            "1:",
            "mfspr 3,{L2CR}",
            "clrlwi 0,3,31",
            "cmplwi 0,0x0000",
            "bne 1b",
            "mfspr 3,{L2CR}",
            "rlwinm 3,3,0,11,9",
            "mtspr {L2CR},3",
            "2:",
            "mfspr 3,{L2CR}",
            "clrlwi 0,3,31",
            "cmplwi 0,0x0000",
            "bne 2b",
            "lwz 0,12(1)",
            "addi 1,1,8",
            "mtlr 0",
            "blr",
            L2CR = const 1017,
            l2_disable = sym l2_disable,
            options(noreturn)
        )
    }
}
