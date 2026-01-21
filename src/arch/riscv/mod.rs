use core::arch::asm;

mod boot;

pub fn wfi() {
    unsafe {
        asm!("wfi");
    }
}
