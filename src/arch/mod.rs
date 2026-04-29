pub mod riscv;

use core::fmt::Write;

#[cfg(target_arch = "riscv64")]
pub use riscv::*;

pub struct ConsoleWriter;
impl Write for ConsoleWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print_str(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!($crate::arch::ConsoleWriter, $($arg)*);
    }};
}

#[macro_export]
macro_rules! println {
    () => ($crate::arch::print!("\n"));
    ($($arg:tt)*) => {{
        $crate::arch::print!($($arg)*);
        $crate::arch::print!("\n");
    }};
}

pub use {print, println};

pub enum ResetType {
    Shutdown,
    ColdReboot,
    WarmReboot,
}

pub enum ResetReason {
    NoReason,
    SystemFailure,
}
