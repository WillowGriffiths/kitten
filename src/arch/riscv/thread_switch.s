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

    mv tp, a0

    ld x1, 0(tp)
    ld x2, 8(tp)
    ld x3, 16(tp)
    // skip tp
    ld x5, 32(tp)
    ld x6, 40(tp)
    ld x7, 48(tp)
    ld x8, 56(tp)
    ld x9, 64(tp)
    ld x10, 72(tp)
    ld x11, 80(tp)
    ld x12, 88(tp)
    ld x13, 96(tp)
    ld x14, 104(tp)
    ld x15, 112(tp)
    ld x16, 120(tp)
    ld x17, 128(tp)
    ld x18, 136(tp)
    ld x19, 144(tp)
    ld x20, 152(tp)
    ld x21, 160(tp)
    ld x22, 168(tp)
    ld x23, 176(tp)
    ld x24, 184(tp)
    ld x25, 192(tp)
    ld x26, 200(tp)
    ld x27, 208(tp)
    ld x28, 216(tp)
    ld x29, 224(tp)
    ld x30, 232(tp)
    ld x31, 240(tp)

    sret
