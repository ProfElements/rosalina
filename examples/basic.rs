#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use core::panic::PanicInfo;

core::arch::global_asm! {
"
  .global __start
  __start:
    b asm_init

  asm_init:
    

  call_to_rust_main: 
    b main
  end_of_init_code:
    ",
options(raw)
}

#[no_mangle]
extern "C" fn main() -> ! {
    loop {}
}

#[panic_handler]
fn panic_handler(_: &PanicInfo) -> ! {
    loop {}
}
