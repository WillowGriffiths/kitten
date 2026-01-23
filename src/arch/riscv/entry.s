.section .text.boot

.set PTE_V, (1L << 0) // valid
.set PTE_R, (1L << 1) // readable
.set PTE_W, (1L << 2) // writable
.set PTE_X, (1L << 3) // executable
.set PTE_U, (1L << 4) // user can access

// fn _start(hart_id: u64, fdt_addr: *const u8) -> !
.global _start
.type _start, @function
_start:
    mv s0, a0
    mv s1, a1

    call early_setup

    mv a0, s0
    mv a1, s1

    la a2, __kernel_start
    la a3, __kernel_end

    lui sp, %hi(__stack_top)
    addi sp, sp, %lo(__stack_top)

    lui t0, %hi(rust_entry)
    jalr x0,%lo(rust_entry)(t0)

1:  wfi
    j 1b

// fn early_setup() -> void
.type early_setup, @function
early_setup:
    li t0, 0 // t0 = i
    li t1, 512
    la t2, __boot_page_table
    
1:  sd x0, 0(t2)
    
    addi t0,t0,1
    addi t2,t2,8
    bne t0, t1, 1b

    la t6, __boot_page_table

    // t0 = &__boot_page_table + boot level 2 index * 8
    la t0, __boot_start
    srli t0, t0, 30
    andi t0, t0, 0x1ff
    slli t0, t0, 3
    add t0, t0, t6

    // t1 = &__boot_page_table + virtual memory level 2 index * 8
    lui t1, %hi(__virtual_start)
    addi t1, t1, %lo(__virtual_start)
    srli t1, t1, 30
    andi t1, t1, 0x1ff
    slli t1, t1, 3
    add t1, t1, t6

    // create page table entry for kernel
    la t2, __boot_start
    srli t2, t2, 30
    slli t2, t2, 28
    ori t2, t2, (PTE_V | PTE_R | PTE_W | PTE_X)

    sd t2, 0(t0)
    sd t2, 0(t1)

    // and enable paging! 🎉

    sfence.vma zero,zero

    li t0, 8
    slli t0, t0, 60
    srli t1, t6, 12
    or t0, t0, t1

    csrw satp, t0

    sfence.vma zero,zero
    
    ret

