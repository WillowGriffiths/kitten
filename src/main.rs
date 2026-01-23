#![no_std]
#![no_main]
#[feature(allocator_api)]
mod arch;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let message = info.message();
    println!("panic: {message:?}");

    loop {}
}

pub fn main(boot_info: arch::BootInfo) -> ! {
    arch::println!("Hello world! :)");
    arch::println!("{:#x?}", boot_info);

    loop {
        arch::wfi();
    }
}
