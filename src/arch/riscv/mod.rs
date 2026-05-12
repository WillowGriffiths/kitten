use core::arch::asm;

pub mod boot;
pub mod pagetable;

mod sbi;
pub use sbi::*;

pub fn wfi() {
    unsafe {
        asm!("wfi");
    }
}
