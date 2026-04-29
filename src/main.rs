#![no_std]
#![no_main]
#![feature(allocator_api)]

mod allocator;
mod arch;
mod device_tree;
mod memory;

use core::panic::PanicInfo;

use alloc::vec;

use crate::arch::boot::BootInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let message = info.message();
    if let Some(location) = info.location() {
        println!(
            "panic at {}:{}: {message:?}",
            location.file(),
            location.line()
        );
    } else {
        println!("panic: {message:?}");
    }

    loop {
        arch::wfi();
    }
}

pub fn main(boot_info: BootInfo) -> ! {
    allocator::setup(&boot_info);
    println!("Hello world! :)");

    loop {
        arch::wfi();
    }
}
