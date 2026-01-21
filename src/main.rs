#![no_std]
#![no_main]

mod arch;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

pub fn main() -> ! {
    loop {
        arch::wfi();
    }
}
