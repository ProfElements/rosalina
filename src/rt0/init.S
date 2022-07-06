  .globl __start
  __start:
    b asm_init

  asm_init:
    bl init_bats
    bl init_gprs
    bl init_hardware
    bl init_system
    
    lis 3,sbss_start@h
    ori 3,3,sbss_start@l
    li 4,0
    lis 5,sbss_end@h
    ori 5,5,sbss_end@l
    sub 5,5,3
    bl _memset

    lis 3,bss_start@h
    ori 3,3,bss_start@l
    li 4,0
    lis 5,bss_end@h
    ori 5,5,bss_end@l
    sub 5,5,3
    bl _memset



    b main
  
  .globl init_bats 
  init_bats:
    mflr 31 //r31
    oris 31,31,0x8000 //r31
    lis 3,config_bats@h //r3
    ori 3,3,config_bats@l //r3
    bl real_mode
    mtlr 31 //r31
    blr
  
  .globl real_mode
  real_mode:
    clrlwi 3,3,2 //r3 //r3
    mtsrr0 3 //r3
    mfmsr 3 //r3
    rlwinm 3,3,0,28,25 //r3
    mtsrr1 3 //r3
    
    rfi

  .globl config_bats
  config_bats:
    lis 3,0x0011 //r3
    ori 3,3,0x0c64 //r3
    mtspr 1008,3 //HID0 //r3
    isync

    lis 3,0x8200 //r3
    mtspr 1011,3 //HID4 //r3
    isync

    li 0,0 //r0
    mtspr 528,0; mtspr 530,0; mtspr 532,0; mtspr 534,0;  //IBAT0U-IBAT3U //r0
    mtspr 536,0; mtspr 538,0; mtspr 540,0; mtspr 542,0;  //DBAT0U-DBAT3U //r0
    mtspr 560,0; mtspr 562,0; mtspr 564,0; mtspr 566,0;  //IBAT4U-IBAT7U //r0
    mtspr 568,0; mtspr 570,0; mtspr 572,0; mtspr 574,0;  //DBAT4U-DBAT7U //r0
    isync

    lis 0,0x8000 //r0 
    mtsr 0,0; mtsr 1,0; mtsr 2,0; mtsr 3,0; mtsr 4,0;      //r0 
    mtsr 5,0; mtsr 6,0; mtsr 7,0; mtsr 8,0; mtsr 9,0;      //r0
    mtsr 10,0; mtsr 11,0; mtsr 12,0; mtsr 13,0; mtsr 14,0; //r0
    mtsr 15,0 //r0
    isync

    //IBAT + DBAT 0 = 80000000 with 256 MB
    li 3,2 //r3
    lis 4,0x8000 //r4
    ori 4,4,0x1fff //r4
    mtspr 529,3 //IBATOL //r3
    mtspr 528,4 //IBAT0U //r4
    mtspr 537,3 //DBAT0L //r3
    mtspr 536,4 //DBAT0U //r4
    isync


    //IBAT + DBAT 4 = 90000000 with 256 MB
    addis 3,3,0x1000 //r3
    addis 4,4,0x1000 //r4
    mtspr 561,3 //r3
    mtspr 560,4 //r4
    mtspr 569,3 //r3
    mtspr 568,4 //r4
    isync 

    //DBAT  1 = C0000000 with 256 MB
    li 3, 0x2a // r3
    lis 4,0xc000 //r4
    ori 4,4,0x1fff //r4
    mtspr 539,3 //r3
    mtspr 538,4 //r4
    isync

    //DBAT 5 = D00000000 with 256 MB
    addi 3,3,0x1000 //r3
    addis 4,4,0x1000 //r4
    mtspr 571,3
    mtspr 570,4
    isync

    mfmsr 3 //r3
    ori 3,3,0x00000010|0x00000020 //Machine state register enable data and instruction data translation stuff 
    mtsrr1 3 //r3
    mflr 3 //r3
    oris 3,3,0x8000 //r3
    mtsrr0 3 //r3
    rfi

    init_gprs:
    li 0,0
    li 3,0
    li 4,0
    li 5,0
    li 6,0
    li 7,0
    li 8,0
    li 9,0
    li 10,0
    li 11,0
    li 12,0
    li 13,0
    li 14,0
    li 15,0
    li 16,0
    li 17,0
    li 18,0
    li 19,0
    li 20,0
    li 21,0
    li 22,0
    li 23,0
    li 24,0
    li 25,0
    li 26,0
    li 27,0
    li 28,0
    li 29,0
    li 30,0
    li 31,0

    lis 1,0x8173 //sp
    ori 1,1,0xfff0 //sp
    addi 1,1,-4 //sp
    stw 0, 0(1) //r0 //sp
    stwu 1, -56(1) //sp //sp
    
    lis 2,SDA2_BASE@h //r2
    ori 2,2,SDA2_BASE@l //r2
    lis 13,SDA_BASE@h //13
    ori 13,13,SDA_BASE@l //r13
    blr 

  init_hardware:
    mfmsr 3 //r3
    ori 3,3,0x00002000 //r3
    mtmsr 3 //r3
    
    mflr 31
    bl init_ps
    bl init_fprs
    //bl init_cache
    mtlr 31 //r31
    
    blr
  init_system:
    mflr 0 //r0
    stw 0,4(1) //r0 //sp
    stwu 1,-24(1) //sp //sp
    stmw 29,12(1) //r29 //sp
    mfmsr 3
    rlwinm 4,3,0,17,15 //r4 //r3
    rlwinm 4,4,0,26,24 //r4 //r4
    mtmsr 4 //r4

    li 3,0 //r3
    mtspr 952,3 //MMCR0 //r3
    mtspr 956,3 //MMCR1 //r3
    mtspr 953,3 //PMC1 //r3
    mtspr 954,3 //PMC2 //r3
    mtspr 957,3 //PMC3 //r3
    mtspr 958,3 //PMC4 //r3
    isync

    mfspr 3,1011 //r3 //HID4
    ori 3,3,0x0190 //r3 //r3     
    mtspr 1011,3 //HID4 //r3
    isync 

    mfspr 3,1008 //r3 //HID0
    ori 3,3,0x0200 //r3 //r3  //BUS ACCESS NO SPECULATION SPECULATION BAD
    mtspr 1008,3 //HID0 //r3
    sync

    mtfsb1 29 //??? Non IEEEE blah float mode thing
     
    mfspr 3,920 //r3 //HID2
    rlwinm 3,3,0,2,0 //r3 //r3 //GX PIPE THINGY OFF
    mtspr 920,3 //HID2 //r3
    isync
    
    lwz 0,28(1) //r0 //sp
    lmw 29,12(1) //r29 //sp
    addi 1,1,24 //sp //sp
    mtlr 0 //r0
    blr

end_of_init_code:
.globl _memset
_memset:
  blr
  clrlwi. 6,5,29 //r6 //r5
  srwi 5,5,2 //r5 //r5
  subi 3,3,4 //r3 //r3
  mtctr 5 //r5
1:stwu 4,4(3) //r4 //r3
  bdnz 1b
  cmplwi 6,0
  beq 3f
2:stbu 4,1,(3) //r4 //r3
  addic. 6,6,-1 //r6 //r6
  bne+ 2b
3:blr
