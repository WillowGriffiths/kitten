pub mod riscv;

use core::fmt::Write;

#[cfg(target_arch = "riscv64")]
pub use riscv::*;

pub struct ConsoleWriter {
    _private: (),
}

impl Write for ConsoleWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print_str(s);
        Ok(())
    }
}

pub static CONSOLE_WRITER: SpinLock<ConsoleWriter> = SpinLock::new(ConsoleWriter { _private: () });

use crate::sync::SpinLock;

#[allow(dead_code)]
pub enum ResetType {
    Shutdown,
    ColdReboot,
    WarmReboot,
}

#[allow(dead_code)]
pub enum ResetReason {
    NoReason,
    SystemFailure,
}
