use core::arch::asm;

pub mod boot;

mod sbi;
pub use sbi::print_str;

pub fn wfi() {
    unsafe {
        asm!("wfi");
    }
}
