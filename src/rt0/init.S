
  .global __start
  __start:
    b asm_init

  asm_init:
    bl init_bats
  call_to_rust_main: 
    b main
  
  init_bats:
    mflr	31
	  oris	31,31,0x8000
	  lis		3,config_bats@h
	  ori		3,3,config_bats@l
	  bl		real_mode
	  mtlr	31
	  blr

  config_bats:
    lis 3,0x0011
    ori 3,3,0x0c64
    mtspr 1008,3 
    isync
    lis 3,0x8200
    mtspr 1011,3
    isync

  real_mode:
    clrlwi 3,3,2
    mtsrr0 3
    mfmsr 3
    rlwinm 3,3,0,28,25
    mtsrr1 3
    rfi
  
  end_of_init_code:

