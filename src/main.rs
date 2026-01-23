#![no_std]
#![no_main]
#![feature(allocator_api)]

mod arch;
mod device_tree;
mod memory;

use core::panic::PanicInfo;

use crate::arch::boot::BootInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let message = info.message();
    println!("panic: {message:?}");

    loop {
        arch::wfi();
    }
}

pub fn main(boot_info: BootInfo) -> ! {
    println!("Hello world! :)");

    loop {
        arch::wfi();
    }
}
