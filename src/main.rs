#![no_std]
#![no_main]
#![feature(allocator_api)]

mod arch;

use core::panic::PanicInfo;

use crate::arch::device_tree::{KernelMapping, MemoryRange};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let message = info.message();
    println!("panic: {message:?}");

    loop {}
}

pub fn main(boot_info: arch::device_tree::BootInfo) -> ! {
    arch::println!("Hello world! :)");
    arch::println!("{:#x?}", boot_info);

    let kernel_virtual: MemoryRange<KernelMapping> = boot_info.kernel_memory.into();

    arch::println!("{:#x?}", kernel_virtual);

    loop {
        arch::wfi();
    }
}
