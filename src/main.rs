#![no_std]
#![no_main]
#![feature(allocator_api)]
#![feature(ptr_alignment_type)]

extern crate alloc;

mod allocator;
mod arch;
mod device_tree;
mod memory;

use core::panic::PanicInfo;

use alloc::vec;

use crate::arch::boot::BootInfo;

const BOOT_MESSAGE: &str = include_str!("./boot_message.txt");

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

    arch::reset(arch::ResetType::Shutdown, arch::ResetReason::SystemFailure);

    loop {
        arch::wfi();
    }
}

pub fn main(boot_info: BootInfo) -> ! {
    print!("{BOOT_MESSAGE}");

    allocator::setup(&boot_info);

    let things = vec!["thing 1", "thing 2", "thing 3"];

    println!("We just heap allocated some things: {things:?}");

    println!("Expecting a panic now!");
    // will fail; deallocation is unimplemented
    drop(things);

    loop {
        arch::wfi();
    }
}
