.section .text

.global _trap_entry
.type _trap_entry, @function
_trap_entry:
    // a0 = &ctx, sscratch = user's a0
    csrrw a0, sscratch, a0

    // store t0 to scratch space
    sd t0, 8(a0)

    // retrieve ctx.current_task
    ld t0, 16(a0)

    // a0 = user's a0, sscratch = &ctx
    csrrw a0, sscratch, a0

    // save all general-purpose registers
    sd ra, 0(t0)
    sd sp, 8(t0)
    sd gp, 16(t0)
    sd tp, 24(t0)
    // skip t0
    sd t1, 40(t0)
    sd t2, 48(t0)
    sd s0, 56(t0)
    sd s1, 64(t0)
    sd a0, 72(t0)
    sd a1, 80(t0)
    sd a2, 88(t0)
    sd a3, 96(t0)
    sd a4, 104(t0)
    sd a5, 112(t0)
    sd a6, 120(t0)
    sd a7, 128(t0)
    sd s2, 136(t0)
    sd s3, 144(t0)
    sd s4, 152(t0)
    sd s5, 160(t0)
    sd s6, 168(t0)
    sd s7, 176(t0)
    sd s8, 184(t0)
    sd s9, 192(t0)
    sd s10, 200(t0)
    sd s11, 208(t0)
    sd t3, 216(t0)
    sd t4, 224(t0)
    sd t5, 232(t0)
    sd t6, 240(t0)

    // a0 = &ctx
    csrr a0, sscratch

    // store t0 from scratch space
    ld t1, 8(a0)
    sd t1, 32(t0)

    // store previous pc
    csrr t1, sepc
    sd t1, 248(t0)

    // load interrupt stack
    ld sp, 0(a0)

    call rust_trap
