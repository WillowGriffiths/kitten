ENTRY(_start)

SECTIONS
{
    . = 0x80000000;
    __boot_start = .;
    . += 0x1000000;


    .boot : {
        *(.text.boot)
        *(.text.boot*)

        . = ALIGN(4K);
        __boot_page_table = .;
        . += 4K;
    }

    __boot_end = .;

    __boot_size = . - __boot_start;

    . = 0xffffffff80000000;
    __virtual_start = .;
    DIFF = . - __boot_start;
    . += __boot_size;

    __kernel_start = . - DIFF;
    __virtual_kernel_start = .;

    .text : AT(. - DIFF) {
        *(.text*) 
    }

    .rodata : AT(. - DIFF) {
        . = ALIGN(8);
        *(.rodata*)
        . = ALIGN(8);
    }
    .data : AT(. - DIFF) { 
        . = ALIGN(8);
        *(.data*)
        . = ALIGN(8);
    } 
    .bss : AT(. - DIFF) {
        . = ALIGN(8);
        _bss_start = .;
        *(.bss*)
        *(COMMON)
        . = ALIGN(8);
        _bss_end = .;
    }

    .kstack : AT(. - DIFF) {
        . = ALIGN(16);
        . += 4K * 8;
        __stack_top = .;
    }

    . = ALIGN(4K);
    __virtual_end = .;

    __kernel_end = . - DIFF;

    /DISCARD/ : { *(.comment .note .eh_frame) }
}
