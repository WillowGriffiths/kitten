#![no_std]
#![no_main]
#![feature(allocator_api)]
#![feature(ptr_alignment_type)]

extern crate alloc;

mod allocator;
mod arch;
mod device_tree;
mod memory;
mod smt;
mod sync;

use core::{fmt::Write, panic::PanicInfo};

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

            let mut writer = arch::CONSOLE_WRITER.lock();

            _ = writeln!(
                writer,
                "{}:{} {color_code}{}{color_reset} - {}",
                record.file().unwrap(),
                record.line().unwrap(),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let message = info.message();
    let mut writer = arch::CONSOLE_WRITER.lock();

    if let Some(location) = info.location() {
        _ = writeln!(
            writer,
            "panic at {}:{}: {message:?}",
            location.file(),
            location.line()
        );
    } else {
        _ = writeln!(writer, "panic: {message:?}");
    }

    arch::reset(arch::ResetType::Shutdown, arch::ResetReason::SystemFailure);
}

pub fn secondary_main(boot_res: smt::BootRes) -> ! {
    log::info!("hello from cpu {}", boot_res.cpu_id);

    loop {
        arch::wfi();
    }
}

pub fn main(boot_info: BootInfo) -> ! {
    let mut writer = arch::CONSOLE_WRITER.lock();
    _ = write!(writer, "{BOOT_MESSAGE}");
    drop(writer);

    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Info))
        .unwrap();

    log::info!("booting on cpu {}", boot_info.boot_cpu);

    allocator::setup(&boot_info);
    smt::init(&boot_info);

    log::info!("free ram: {}", allocator::free_ram());

    loop {
        arch::wfi();
    }
}
