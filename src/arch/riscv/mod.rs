use core::arch::asm;

pub mod boot;
mod pagetable;

mod sbi;
pub use sbi::{print_str, reset};

pub fn wfi() {
    unsafe {
        asm!("wfi");
    }
}
