use core::{mem::MaybeUninit, ptr::slice_from_raw_parts_mut};

use crate::os::LinkerSymbol;

#[no_mangle]
#[naked]
unsafe extern "C" fn __start() -> ! {
    core::arch::asm!(
        "bl {__init_bats}",
        "bl {__init_gprs}",
        "bl {__init_hardware}",
        "bl {__init_system}",
        "bl {__clear_statics}",
        "b main",
        __init_bats = sym __init_bats,
        __init_gprs = sym __init_gprs,
        __init_hardware = sym __init_hardware,
        __init_system = sym __init_system,
        __clear_statics = sym __clear_statics,
        options(noreturn)
    )
}

#[no_mangle]
#[naked]
unsafe extern "C" fn __init_bats() -> ! {
    core::arch::asm!(
        "mflr 31",
        "oris 31,31,0x8000",
        "lis 3,{__config_bats}@h",
        "ori 3,3,{__config_bats}@l",
        "bl {__real_mode}",
        "mtlr 31",
        "blr",
        __config_bats = sym __config_bats,
        __real_mode = sym __real_mode,
        options(noreturn)
    )
}

#[no_mangle]
#[naked]
unsafe extern "C" fn __config_bats() -> ! {
    core::arch::asm!(
        "lis 3,0x0011",
        "ori 3,3,0x0c64",
        "mtspr {HID0},3",
        "isync",
        "lis 3,0x8200",
        "mtspr {HID4},3",
        "isync",
        "li 0,0",
        "mtspr {IBAT0U},0",
        "mtspr {IBAT1U},0",
        "mtspr {IBAT2U},0",
        "mtspr {IBAT3U},0",
        "mtspr {DBAT0U},0",
        "mtspr {DBAT1U},0",
        "mtspr {DBAT2U},0",
        "mtspr {DBAT3U},0",
        "mtspr {IBAT4U},0",
        "mtspr {IBAT5U},0",
        "mtspr {IBAT6U},0",
        "mtspr {IBAT7U},0",
        "mtspr {DBAT4U},0",
        "mtspr {DBAT5U},0",
        "mtspr {DBAT6U},0",
        "mtspr {DBAT7U},0",
        "isync",
        "lis 0,0x8000",
        "mtsr 0,0",
        "mtsr 1,0",
        "mtsr 2,0",
        "mtsr 3,0",
        "mtsr 4,0",
        "mtsr 5,0",
        "mtsr 6,0",
        "mtsr 7,0",
        "mtsr 8,0",
        "mtsr 9,0",
        "mtsr 10,0",
        "mtsr 11,0",
        "mtsr 12,0",
        "mtsr 13,0",
        "mtsr 14,0",
        "mtsr 15,0",
        "isync",
        "li 3,2",
        "lis 4,0x8000",
        "ori 4,4,0x1FFF",
        "mtspr {IBAT0L},3",
        "mtspr {IBAT0U},4",
        "mtspr {DBAT0L},3",
        "mtspr {DBAT0U},4",
        "isync",
        "addis 3,3,0x1000",
        "addis 4,4,0x1000",
        "mtspr {IBAT4L},3",
        "mtspr {IBAT4U},4",
        "mtspr {DBAT4L},3",
        "mtspr {DBAT4U},4",
        "isync",
        "li 3,0x2a",
        "lis 4,0xC000",
        "ori 4,4,0x1FFF",
        "mtspr {DBAT1L},3",
        "mtspr {DBAT1U},4",
        "isync",
        "addis 3,3,0x1000",
        "addis 4,4,0x1000",
        "mtspr {DBAT5L},3",
        "mtspr {DBAT5U},4",
        "isync",
        "mfmsr 3",
        "ori 3,3,{MSR_DR}|{MSR_IR}",
        "mtsrr1 3",
        "mflr 3",
        "oris 3,3,0x8000",
        "mtsrr0 3",
        "rfi",
        HID0 = const 1008,
        HID4 = const 1011,
        IBAT0U = const 528,
        IBAT1U = const 530,
        IBAT2U = const 532,
        IBAT3U = const 534,
        IBAT4U = const 560,
        IBAT5U = const 562,
        IBAT6U = const 564,
        IBAT7U = const 566,
        DBAT0U = const 536,
        DBAT1U = const 538,
        DBAT2U = const 540,
        DBAT3U = const 542,
        DBAT4U = const 568,
        DBAT5U = const 570,
        DBAT6U = const 572,
        DBAT7U = const 574,
        IBAT0L = const 529,
        DBAT0L = const 537,
        IBAT4L = const 561,
        DBAT4L = const 569,
        DBAT1L = const 539,
        DBAT5L = const 571,
        MSR_DR = const 0x10,
        MSR_IR = const 0x20,
        options(noreturn)
    )
}

#[no_mangle]
#[naked]
unsafe extern "C" fn __real_mode() -> ! {
    core::arch::asm!(
        "clrlwi 3,3,2",
        "mtsrr0 3",
        "mfmsr 3",
        "rlwinm 3,3,0,28,25",
        "mtsrr1 3",
        "rfi",
        options(noreturn)
    )
}

#[no_mangle]
#[naked]
unsafe extern "C" fn __init_gprs() -> ! {
    core::arch::asm!(
        "li 0,0",
        "li 3,0",
        "li 4,0",
        "li 5,0",
        "li 6,0",
        "li 7,0",
        "li 8,0",
        "li 9,0",
        "li 10,0",
        "li 11,0",
        "li 12,0",
        "li 13,0",
        "li 14,0",
        "li 15,0",
        "li 16,0",
        "li 17,0",
        "li 18,0",
        "li 19,0",
        "li 20,0",
        "li 21,0",
        "li 22,0",
        "li 23,0",
        "li 24,0",
        "li 25,0",
        "li 26,0",
        "li 27,0",
        "li 28,0",
        "li 29,0",
        "li 30,0",
        "li 31,0",
        "lis 1,0x8173",
        "ori 1,1,0xFFF0",
        "addi 1,1,-4",
        "stw 0,0(1)",
        "stwu 1,-56(1)",
        "lis 2,SDA2_BASE@h",
        "ori 2,2,SDA2_BASE@l",
        "lis 13,SDA_BASE@h",
        "ori 13,13,SDA_BASE@l",
        "blr",
        options(noreturn)
    )
}

#[no_mangle]
#[naked]
unsafe extern "C" fn __init_hardware() -> ! {
    core::arch::asm!(
        "mfmsr 3",
        "ori 3,3,{MSR_FP}",
        "mtmsr 3",
        "mflr 31",
        "bl {__init_ps}",
        "bl {__init_fprs}",
        "bl {__init_cache}",
        "mtlr 31",
        "blr",
        MSR_FP = const 0x2000,
        __init_ps = sym __init_ps,
        __init_fprs = sym __init_fprs,
        __init_cache = sym crate::cache::__init_cache,
        options(noreturn)
    )
}

#[no_mangle]
#[naked]
unsafe extern "C" fn __init_ps() -> ! {
    core::arch::asm!(
        "mflr 0",
        "stw 0,4(1)",
        "stwu 1,-8(1)",
        "mfspr 3,{HID2}",
        "oris 3,3,0xA000",
        "mtspr {HID2},3",
        "isync",
        "bl {ic_flash_invalidate}",
        "sync",
        "li 3,0",
        "mtspr {GQR0},3",
        "mtspr {GQR1},3",
        "mtspr {GQR2},3",
        "mtspr {GQR3},3",
        "mtspr {GQR4},3",
        "mtspr {GQR5},3",
        "mtspr {GQR6},3",
        "mtspr {GQR7},3",
        "isync",
        "lwz 0,12(1)",
        "addi 1,1,8",
        "mtlr 0",
        "blr",
        HID2 = const 920,
        GQR0 = const 912,
        GQR1 = const 913,
        GQR2 = const 914,
        GQR3 = const 915,
        GQR4 = const 916,
        GQR5 = const 917,
        GQR6 = const 918,
        GQR7 = const 919,
        ic_flash_invalidate = sym crate::cache::ic_flash_invalidate,
        options(noreturn)
    )
}

#[no_mangle]
#[naked]
#[allow(named_asm_labels)]
unsafe extern "C" fn __init_fprs() -> ! {
    core::arch::asm!(
        "mfmsr 3",
        "ori 3,3,{MSR_FP}",
        "mtmsr 3",
        "mfspr 3,{HID2}",
        "extrwi. 3,3,1,2",
        "lis 3,zeroF@ha",
        "lfd 0,zeroF@l(3)",
        "fmr 1,0",
        "fmr 2,0",
        "fmr 3,0",
        "fmr 4,0",
        "fmr 5,0",
        "fmr 6,0",
        "fmr 7,0",
        "fmr 8,0",
        "fmr 9,0",
        "fmr 10,0",
        "fmr 11,0",
        "fmr 12,0",
        "fmr 13,0",
        "fmr 14,0",
        "fmr 15,0",
        "fmr 16,0",
        "fmr 17,0",
        "fmr 18,0",
        "fmr 19,0",
        "fmr 20,0",
        "fmr 21,0",
        "fmr 22,0",
        "fmr 23,0",
        "fmr 24,0",
        "fmr 25,0",
        "fmr 26,0",
        "fmr 27,0",
        "fmr 28,0",
        "fmr 29,0",
        "fmr 30,0",
        "fmr 31,0",
        "mtfsf 255,0",
        "blr",
        ".global",
        "zeroF:",
        ".double 0.0",
        MSR_FP = const 0x2000,
        HID2 = const 920,
        options(noreturn)
    )
}

#[no_mangle]
#[naked]
unsafe extern "C" fn __init_system() -> ! {
    core::arch::asm!(
        "mflr 0",
        "stw 0,4(1)",
        "stwu 1,-24(1)",
        "stmw 29,-24(1)",
        "mfmsr 3",
        "rlwinm 4,3,0,17,15",
        "rlwinm 3,4,0,26,24",
        "mtmsr 4",
        "li 3,0",
        "mtspr {MMCR0},3",
        "mtspr {MMCR1},3",
        "mtspr {PMC1},3",
        "mtspr {PMC2},3",
        "mtspr {PMC3},3",
        "mtspr {PMC4},3",
        "isync",
        "mfspr 3,{HID4}",
        "ori 3,3,0x190",
        "mtspr {HID4},3",
        "isync",
        "mfspr 3,{HID0}",
        "ori 3,3,0x200",
        "mtspr {HID0},3",
        "isync",
        "mtfsb1 29",
        "mfspr 3,{HID2}",
        "rlwinm 3,3,0,2,0",
        "mtspr {HID2},3",
        "isync",
        "lwz 0,28(1)",
        "lmw 29,12(1)",
        "addi 1,1,24",
        "mtlr 0",
        "blr",
        HID0 = const 1008,
        HID2 = const 920,
        HID4 = const 1011,
        MMCR0 = const 952,
        MMCR1 = const 956,
        PMC1 = const 953,
        PMC2 = const 954,
        PMC3 = const 957,
        PMC4 = const 958,

        options(noreturn)
    )
}

#[no_mangle]
unsafe extern "C" fn __clear_statics() {
    extern "C" {
        static __bss_start: LinkerSymbol;
        static __bss_end: LinkerSymbol;
        static __sbss_start: LinkerSymbol;
        static __sbss_end: LinkerSymbol;
    }

    slice_from_raw_parts_mut(
        __bss_start.as_mut_ptr().cast::<MaybeUninit<u8>>(),
        __bss_end.as_usize() - __bss_start.as_usize(),
    )
    .as_mut()
    .unwrap()
    .fill(MaybeUninit::new(0x0));

    slice_from_raw_parts_mut(
        __sbss_start.as_mut_ptr().cast::<MaybeUninit<u8>>(),
        __sbss_end.as_usize() - __sbss_start.as_usize(),
    )
    .as_mut()
    .unwrap()
    .fill(MaybeUninit::new(0x0));
}
