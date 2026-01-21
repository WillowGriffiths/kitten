use core::arch::asm;

mod boot;
mod sbi;

pub use sbi::print_str;

pub fn wfi() {
    unsafe {
        asm!("wfi");
    }
}
