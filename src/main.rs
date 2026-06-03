#![no_std]
#![no_main]
#![feature(allocator_api)]
#![feature(ptr_alignment_type)]

extern crate alloc;

mod allocator;
mod arch;
mod device_tree;
mod interrupts;
mod memory;
mod scheduler;
mod smp;
mod sync;

use core::{cell::UnsafeCell, fmt::Write, panic::PanicInfo};

use crate::{
    arch::boot::BootInfo,
    device_tree::{FdtInfo, FdtNode, FdtNodeChild},
    smp::BootRes,
};

const BOOT_MESSAGE: &str = include_str!("./boot_message.txt");

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
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
                "[{} {color_code}{}{color_reset}] {}",
                record.target(),
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

#[allow(clippy::large_enum_variant)]
pub enum BootData {
    Primary(BootInfo),
    Secondary(BootRes),
}

// assume a shared timebase frequency.
fn parse_cpus(node: FdtNode) -> Option<usize> {
    let mut timebase_frequency = None;

    for child in node {
        match child {
            FdtNodeChild::Node(_node) => {}
            FdtNodeChild::Prop(name, data) => {
                if name == "timebase-frequency" {
                    if timebase_frequency.is_some() {
                        panic!("timebase-frequency defined twice");
                    }

                    if data.len() == 4 {
                        timebase_frequency =
                            Some(u32::from_be_bytes(data[0..4].try_into().unwrap()) as usize);
                    } else if data.len() == 8 {
                        timebase_frequency =
                            Some(u64::from_be_bytes(data[0..8].try_into().unwrap()) as usize);
                    }
                }
            }
        }
    }

    timebase_frequency
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DeviceTreeInfo {
    timebase_frequency: usize,
}

pub struct GlobalData {
    timebase_frequency: usize,
}

struct GlobalDataContainer(UnsafeCell<Option<GlobalData>>);

unsafe impl Sync for GlobalDataContainer {}

static GLOBAL_DATA: GlobalDataContainer = GlobalDataContainer(UnsafeCell::new(None));

pub fn global_data() -> &'static GlobalData {
    unsafe { (&*GLOBAL_DATA.0.get()).as_ref().unwrap_unchecked() }
}

// boot.rs parses the device tree for certain essential properties. After
// further initialisation, we can do some more sophisticated parsing with the
// power of the heap at our disposal.
fn parse_device_tree(boot_info: &BootInfo) -> DeviceTreeInfo {
    let addr = memory::to_virt(boot_info.fdt_addr as u64) as *const u8;
    let fdt = FdtInfo::new(addr);

    let mut timebase_frequency = None;

    for child in fdt.root_node() {
        match child {
            FdtNodeChild::Node(node) => {
                if node.name == "cpus" {
                    timebase_frequency = parse_cpus(node);
                }
            }
            FdtNodeChild::Prop(_name, _data) => {}
        }
    }

    DeviceTreeInfo {
        timebase_frequency: timebase_frequency.expect("failed to find timebase-frequency"),
    }
}

pub fn main(data: BootData) -> ! {
    let (cpu_id, mut our_stack) = match data {
        BootData::Primary(boot_info) => {
            let mut writer = arch::CONSOLE_WRITER.lock();
            _ = write!(writer, "{BOOT_MESSAGE}");
            drop(writer);

            log::set_logger(&LOGGER)
                .map(|()| log::set_max_level(log::LevelFilter::Info))
                .unwrap();

            log::debug!("{boot_info:#?}");

            log::info!("booting on cpu {}", boot_info.boot_cpu);

            allocator::setup(&boot_info);
            let device_tree_info = parse_device_tree(&boot_info);

            unsafe {
                *GLOBAL_DATA.0.get() = Some(GlobalData {
                    timebase_frequency: device_tree_info.timebase_frequency,
                });
            }

            smp::init(&boot_info, device_tree_info);

            log::info!(
                "timebase frequency: {}Hz",
                device_tree_info.timebase_frequency
            );

            log::info!("free ram: {}", allocator::free_ram());

            (boot_info.boot_cpu as usize, None)
        }
        BootData::Secondary(boot_res) => (boot_res.cpu_id, Some(boot_res.stack)),
    };

    log::trace!("initialising context");

    smp::create_ctx(cpu_id).expect("Failed to initialise cpu context");

    log::trace!("initialising interrupts");

    interrupts::init();

    log::trace!("spawning threads");

    smp::get_ctx()
        .scheduler
        .spawn(move || {
            let cpu_id = smp::get_ctx().cpu_id;

            // drop the old stack
            _ = our_stack.take();

            loop {
                log::info!("hello from cpu {}", cpu_id);

                scheduler::sleep(500);
            }
        })
        .expect("failed to spawn task");

    smp::get_ctx()
        .scheduler
        .spawn(move || {
            let cpu_id = smp::get_ctx().cpu_id;

            loop {
                log::info!("hello again from cpu {}", cpu_id);

                scheduler::sleep(1000);
            }
        })
        .expect("failed to spawn task");

    log::trace!("scheduling");

    smp::get_ctx().scheduler.schedule();

    log::trace!("entering");

    unsafe { (**smp::get_ctx().current_task.get()).enter() };
}
