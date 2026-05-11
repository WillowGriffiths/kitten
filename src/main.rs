#![no_std]
#![no_main]
#![feature(allocator_api)]
#![feature(ptr_alignment_type)]

extern crate alloc;

mod allocator;
mod arch;
mod device_tree;
mod memory;
mod sync;

use core::panic::PanicInfo;

use alloc::{boxed::Box, vec, vec::Vec};

use crate::arch::boot::BootInfo;

const BOOT_MESSAGE: &str = include_str!("./boot_message.txt");

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let color_code = match record.level() {
                log::Level::Error => "\x1b[31m",
                log::Level::Warn => "\x1b[33m",
                log::Level::Info => "\x1b[32m",
                log::Level::Debug => "\x1b[35m",
                log::Level::Trace => "\x1b[36m",
            };

            let color_reset = "\x1b[0m";

            println!(
                "{}:{} {color_code}{}{color_reset} - {}",
                record.file().unwrap(),
                record.line().unwrap(),
                record.level(),
                record.args()
            )
        }
    }

    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

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

    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Debug))
        .unwrap();

    allocator::setup(&boot_info);

    let things = vec!["thing 1", "thing 2", "thing 3"];

    log::info!("We just heap allocated some things: {things:?}");

    let more_things = (0..100_000).map(Box::new).collect::<Vec<_>>();

    log::info!("We just heap allocated some more things!");

    log::info!("Deallocating now!");
    drop(things);
    drop(more_things);

    loop {
        arch::wfi();
    }
}
