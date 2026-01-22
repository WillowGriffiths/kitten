#![no_std]
#![no_main]

mod arch;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let message = info.message();
    println!("panic: {message:?}");

    loop {}
}

pub fn main() -> ! {
    arch::println!("Hello world! :)");

    loop {
        arch::wfi();
    }
}
