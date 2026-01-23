use core::arch::asm;

pub mod boot;
pub mod device_tree;

mod sbi;
pub use sbi::print_str;

pub fn wfi() {
    unsafe {
        asm!("wfi");
    }
}
