.globl init_ps
init_ps:
  mflr 0 //r0
  stw 0,4(1) //r0 //sp
  stwu 1,-8(1) //sp //sp
  mfspr 3,920 //r3 //HID2
  oris 3,3,0xA000 //r3 //r3
  mtspr 920,3 //HID2 //r3
  isync
  bl instruction_cache_flash_invalidate
  sync
  li 3,0 //r3
  mtspr 912,3 //GQR0 //r3
  mtspr 913,3 //GQR1 //r3
  mtspr 914,3 //GQR2 //r3
  mtspr 915,3 //GQR3 //r3
  mtspr 916,3 //GQR4 //r3
  mtspr 917,3 //GQR5 //r3
  mtspr 918,3 //GQR6 //r3
  mtspr 919,3 //GQR7 //r3
  isync
  lwz 0,12(1) //r0 //sp
  addi 1,1,8 //sp //sp
  mtlr 0 //r0
  blr

.globl init_fprs
init_fprs:
  mfmsr 3 //r3
  ori 3,3, 0x00002000 //r3
  mtmsr 3 //r3 
  mfspr 3, 920 //HID2 //r3
  extrwi 3,3,1,2 //r3
  
  //  beq 1f
  //Clear using paired singles
  // lis 3,zeroPS@ha //r3 //r3
  // addi 3,3,zeroPS@l //r3 //r3
  // psq_l 0,0(3),0,0 //fr0 //r3
  // ps_mr 1,0
  // ps_mr 2,0
  // ps_mr 3,0
  // ps_mr 4,0
  // ps_mr 5,0
  // ps_mr 6,0
  // ps_mr 7,0
  // ps_mr 8,0
  // ps_mr 9,0
  // ps_mr 10,0
  // ps_mr 11,0
  // ps_mr 12,0
  // ps_mr 13,0
  // ps_mr 14,0
  // ps_mr 15,0
  // ps_mr 16,0
  // ps_mr 17,0
  // ps_mr 18,0
  // ps_mr 19,0
  // ps_mr 20,0
  // ps_mr 21,0
  // ps_mr 22,0
  // ps_mr 23,0
  // ps_mr 24,0
  // ps_mr 25,0
  // ps_mr 26,0
  // ps_mr 27,0
  // ps_mr 28,0
  // ps_mr 29,0
  // ps_mr 30,0
  // ps_mr 31,0
//Clear using regular float stuff
  lis 3,zeroF@ha //r3
  lfd 0,zeroF@l(3) //fr0 //r3
  fmr 1,0
  fmr 2,0
  fmr 3,0
  fmr 4,0
  fmr 5,0
  fmr 6,0
  fmr 7,0
  fmr 8,0
  fmr 9,0
  fmr 10,0
  fmr 11,0
  fmr 12,0
  fmr 13,0
  fmr 14,0
  fmr 15,0
  fmr 16,0
  fmr 17,0
  fmr 18,0
  fmr 19,0
  fmr 20,0
  fmr 21,0
  fmr 22,0
  fmr 23,0
  fmr 24,0
  fmr 25,0
  fmr 26,0
  fmr 27,0
  fmr 28,0
  fmr 29,0
  fmr 30,0
  fmr 31,0
  mtfsf 255,0 //fr0
  
  blr

.globl init_cache
init_cache:
  mflr 0 //r0
  stw 0,4(1) //r0 //sp
  stwu 1,-16(1) //sp //sp
  stw 31,12(1) //r31 //sp
  mfspr 3,1008 //r3 //HID0
  rlwinm. 0,3,0,16,16 //r0 //r3
  bne instruction_cache_enabled

  bl instruction_cache_enable
zeroPS:
  .float 0.0,0.0

zeroF:
  .double 0.0
