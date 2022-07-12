
.globl instruction_cache_enabled
instruction_cache_enabled:
  mfspr 3,1008 //r3 //HID0
  rlwinm. 0,3,0,17,17 //r0 //r3
  bne data_cache_enabled

  bl data_cache_enable

.globl data_cache_enabled
data_cache_enabled:
  mfspr 3,1017 //r3 //L2CR
  clrrwi. 0,3,31 //r0 //r3
  bne l2_cache_enabled

  bl l2_cache_init
  bl l2_cache_enable

.globl l2_cache_enabled
l2_cache_enabled:
  lwz 0,20(1) //r0 //sp
  lwz 31,12(1) //r31 //sp
  addi 1,1,16 //sp //sp
  mtlr 0 //r0
  blr


.globl instruction_cache_enable
instruction_cache_enable:
  mfspr 3,1008 //r3 //HID0
  ori 3,3,0x8000 //r3 //r3
  mtspr 1008,3 //HID0 //r3
  isync
  blr

.globl data_cache_enable
data_cache_enable:
  mfspr 3,1008 //r3 //HId0
  ori 3,3,0x4000 //r3 //r3
  mtspr 1008,3 //HID0 //r3
  isync
  blr

.globl l2_cache_init
l2_cache_init:
  mflr 0 //r0
  stw 0,4(1) //r0 //sp
  stwu 1,-16(1) //sp //sp
  stw 31,12(1) //r31 //sp
  mfmsr 3 //r3
  mr 31,3 //r31 //r3
  sync
  li 3,48 //r3
  mtmsr 3
  sync
  bl l2_cache_disable
  bl l2_cache_global_invalidate
  mr 3,31 //r3 //r31
  mtmsr 3 //r3
  lwz 0,20(1) //r0 //sp
  lwz 31,12(1) //r31 //sp
  mtlr 0 //r0
  blr


.globl instruction_cache_flash_invalidate
instruction_cache_flash_invalidate:
  mfspr 3,1008 //r3 //HID0
  ori 3,3,0x8000 //r3 //r3 
  mtspr 1008,3 //HID0 //r3
  isync
  blr



.globl l2_cache_enable
l2_cache_enable:
   sync
   mfspr 3,1017 //r3 //L2CR
   ori 0,3,0x8000 //r0 //r3
   rlwinm 3,0,0,11,9 //r3 //r0
   mtspr 1017,3 //L2CR //r3
   sync
   blr

.globl l2_cache_disable
l2_cache_disable:
  sync
  mfspr 3,1017 //r3 //L2CR
  clrlwi 3,3,1 //r3 //r3
  mtspr 1017,3 //L2Cr //r3
  sync
  blr


.globl l2_cache_global_invalidate
l2_cache_global_invalidate:
  mflr 0 //r0
  stw 0,4(1) //r0 //sp
  stwu 1,-8(1) //sp //sp
  bl l2_cache_disable
  mfspr 3,1017 //r3 //L2CR
  ori 3,3,0x020 //r3 //r3
  mtspr 1017,3 //L2CR //r3
1:mfspr 3,1017 //r3 //L2CR
  clrrwi 0,3,31 //r0 //r3
  cmplwi 0,0x0000 //r0
  bne 1b
  mfspr 3,1017 //r3 //L2CR
  rlwinm 3,3,0,11,9 //r3 //r3
  mtspr 1017,3 //L2CR //r3
2:mfspr 3,1017 //r3 //L2CR
  clrlwi 0,3,31 //r0 //r3
  cmplwi 0,0x0000 //r0
  bne 2b
  lwz 0,12(1) //r0 //sp
  addi 1,1,8 //sp /sp
  mtlr 0 //r0
  blr

.globl data_cache_flush_range
data_cache_flush_range:
  cmplwi 4,0 //r4
  blelr
  clrlwi. 5,3,27 //r5 //r3
  beq 1f
  addi 4,4,0x20 //r4 //r4
1:
  addi 4,4,0x1f //r4 //r4
  srwi 4,4,5 //r4 //r4
  mtctr 4 //r4
2:
  dcbf 0,3 //r0 //r3
  addi 3,3,0x20
  bdnz 2b
  sc
  blr
