use core::arch::global_asm;

global_asm!(
    "
    .section .text.boot
    .global _start
    _start:
    1:  wfi
        j 1b
    "
);
