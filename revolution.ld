OUTPUT_FORMAT("elf32-powerpc", "elf32-powerpc", "elf32-powerpc");
OUTPUT_ARCH(powerpc:common);

ENTRY(__start)

SECTIONS {
  . = 0x00003400;
  
  .stub : {
    KEEP(*(.stub));
  } :stub = 0

  . = 0x80004000;

  .text : {
      *(.text)
      *(.text.*)
      . = ALIGN(32);
  }

  .rodata : {
    *(.rodata)
    *(.rodata.*)
    . = ALIGN(32);
  }

  .data : {
    *(.data)
    *(.data.*)
    . = ALIGN(32);
  }

  .bss : {
    __bss_start = .;
    *(.bss)
    *(.bss.*)
    . = ALIGN(32);
    __bss_end = .;
  }   
  .sdata : {
      PROVIDE(SDA_BASE = .);
      *(.sdata)
      *(.sdata.*)
      .  = ALIGN(32);
  }

  .sdata2 : {
      PROVIDE(SDA2_BASE = .);
      *(.sdata2)
      *(.sdata2.*)
      . = ALIGN(32);
  }

  .sbss : {
    __sbss_start = .;
    *(.sbss)
    *(.sbss.*)
    . = ALIGN(32);
    __sbss_end = .; 
  } 

  /* debugging sections */
  /* Stabs */
  .stab            0 : { *(.stab) }
  .stabstr         0 : { *(.stabstr) }
  .stab.excl       0 : { *(.stab.excl) }
  .stab.exclstr    0 : { *(.stab.exclstr) }
  .stab.index      0 : { *(.stab.index) }
  .stab.indexstr   0 : { *(.stab.indexstr) }
  .comment         0 : { *(.comment) }
  /* DWARF 1 */
  .debug           0 : { *(.debug) }
  .line            0 : { *(.line) }
  /* GNU DWARF 1 extensions */
  .debug_srcinfo   0 : { *(.debug_srcinfo) }
  .debug_sfnames   0 : { *(.debug_sfnames) }
  /* DWARF 1.1 and DWARF 2 */
  .debug_aranges   0 : { *(.debug_aranges) }
  .debug_pubnames  0 : { *(.debug_pubnames) }
  /* DWARF 2 */
  .debug_info      0 : { *(.debug_info) }
  .debug_abbrev    0 : { *(.debug_abbrev) }
  .debug_line      0 : { *(.debug_line) }
  .debug_frame     0 : { *(.debug_frame) }
  .debug_str       0 : { *(.debug_str) }
  .debug_loc       0 : { *(.debug_loc) }
  .debug_macinfo   0 : { *(.debug_macinfo) }
  /* SGI/MIPS DWARF 2 extensions */
  .debug_weaknames 0 : { *(.debug_weaknames) }
  .debug_funcnames 0 : { *(.debug_funcnames) }
  .debug_typenames 0 : { *(.debug_typenames) }
  .debug_varnames  0 : { *(.debug_varnames) }

  /* discard anything not already mentioned */
  /DISCARD/ : { *(*) }  
}

__stack_addr = (__bss_start + SIZEOF(.bss) + 0x200000 + 7) & (-8);
__stacK_end = (__bss_start + SIZEOF(.bss));

arena1_lo = ((__stack_addr + 0x4000) + 31) & (-32);
PROVIDE(ARENA_1_LO = arena1_lo);
