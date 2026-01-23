use core::arch::asm;

mod boot;
mod device_tree;
mod sbi;

pub use device_tree::BootInfo;
pub use sbi::print_str;

pub fn wfi() {
    unsafe {
        asm!("wfi");
    }
}
