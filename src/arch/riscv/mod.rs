use core::arch::asm;

pub mod boot;
pub mod pagetable;

mod sbi;
pub mod thread;
pub use sbi::*;

use crate::smp::CpuCtx;

pub fn wfi() {
    unsafe {
        asm!("wfi");
    }
}

pub fn store_ctx(ctx: &'static CpuCtx) {
    unsafe {
        asm!("csrw sscratch, {ptr}", ptr = in(reg) ctx);
    }
}

// safe as long as boot initialises sscratch to 0 and it isn't written to
// anywhere else.
pub fn get_ctx() -> &'static CpuCtx {
    unsafe {
        let addr: usize;
        asm!("csrr {addr}, sscratch", addr = out(reg) addr);
        (addr as *const CpuCtx).as_ref().unwrap()
    }
}
