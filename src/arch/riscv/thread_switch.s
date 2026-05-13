.section .text

// fn thread_switch(ctx: *const ThreadContext) -> !
.global thread_switch
.type thread_switch, @function
thread_switch:
    ld t0, 248(a0)
    csrw sepc, t0

    // set the SPP bit to stay in s-mode
    li t0, 1 << 8
    csrs sstatus, t0

    li t0, 1 << 5
    csrc sstatus, t0

    ld x1, 0(a0)
    ld x2, 8(a0)
    ld x3, 16(a0)
    ld x4, 24(a0)
    ld x5, 32(a0)
    ld x6, 40(a0)
    ld x7, 48(a0)
    ld x8, 56(a0)
    ld x9, 64(a0)
    // skip a0
    ld x11, 80(a0)
    ld x12, 88(a0)
    ld x13, 96(a0)
    ld x14, 104(a0)
    ld x15, 112(a0)
    ld x16, 120(a0)
    ld x17, 128(a0)
    ld x18, 136(a0)
    ld x19, 144(a0)
    ld x20, 152(a0)
    ld x21, 160(a0)
    ld x22, 168(a0)
    ld x23, 176(a0)
    ld x24, 184(a0)
    ld x25, 192(a0)
    ld x26, 200(a0)
    ld x27, 208(a0)
    ld x28, 216(a0)
    ld x29, 224(a0)
    ld x30, 232(a0)
    ld x31, 240(a0)

    ld x10, 72(a0)

    sret
