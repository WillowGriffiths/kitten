use core::arch::asm;

mod boot;
mod sbi;

pub use sbi::print;

pub fn wfi() {
    unsafe {
        asm!("wfi");
    }
}
