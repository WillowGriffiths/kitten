ENTRY(_start)

SECTIONS
{
    . = 0x81000000;

    . = ALIGN(4K);
    __boot_start = .;

    .boot : {
        *(.text.boot)
        *(.text.boot*)

        . = ALIGN(16);
        . += 4K;
        __boot_stack_start = .;

        . = ALIGN(4K);
        __boot_page_tables = .;
        . += 4K * 10;
    }

    __boot_end = .;

    __boot_size = . - __boot_start;

    . = 0xffffffff80000000 + __boot_size;
    DIFF = . - __boot_end;

    __kernel_start = . - DIFF;
    __virtual_start = . - __boot_size;
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
        __stack_start = .;
    }

    . = ALIGN(4K);
    __virtual_end = .;

    __kernel_end = . - DIFF;

    /DISCARD/ : { *(.comment .note .eh_frame) }
}
