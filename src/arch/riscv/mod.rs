use core::arch::asm;

pub mod boot;
pub mod pagetable;

mod sbi;
pub mod thread;
pub use sbi::*;

pub fn wfi() {
    unsafe {
        asm!("wfi");
    }
}
