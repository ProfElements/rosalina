
.global EXCEPTION_HANDLER_START
EXCEPTION_HANDLER_START:
  bl exception_handler
  rfi
.global EXCEPTION_HANDLER_END
EXCEPTION_HANDLER_END:
  nop

.global SYSTEMCALL_HANDLER_START
SYSTEMCALL_HANDLER_START:
  mtspr 274,9 //SPRG2 //r9
  mtspr 275,10 //SPRG3 //r10
  mfspr 9,1008 //r9 //HID0
  ori 10,9,0x0008 //r10 //r9
  mtspr 1008,10
  isync
  sync
  mtspr 1008,9 //HID0 //r9
  mfspr 9,274 //r9 //SPRG2
  mfspr 10,275 //r10 //SPRG3
  rfi
.global SYSTEMCALL_HANDLER_END
SYSTEMCALL_HANDLER_END:
 nop 
