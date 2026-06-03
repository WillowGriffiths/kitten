use core::arch::{asm, global_asm};

use crate::interrupts::InterruptType;

#[allow(dead_code)]
mod consts {
    pub const SIE_STIE: usize = 1 << 5;
    pub const SIE_SSIE: usize = 1 << 1;

    pub const SCAUSE_INTERRUPT_MASK: usize = 1 << 63;
    pub const SCAUSE_CODE_MASK: usize = !SCAUSE_INTERRUPT_MASK;

    pub const SCAUSE_INTR_SUPERVISOR_SOFTWARE: usize = 1;
    pub const SCAUSE_INTR_SUPERVISOR_TIMER: usize = 5;
    pub const SCAUSE_INTR_SUPERVISOR_EXTERNAL: usize = 9;

    pub const SCAUSE_EXCP_INSTRUCTION_MISALIGNED: usize = 0;
    pub const SCAUSE_EXCP_INSTRUCTION_FAULT: usize = 1;
    pub const SCAUSE_EXCP_ILLEGAL_INSTRUCTION: usize = 2;
    pub const SCAUSE_EXCP_BREAKPOINT: usize = 3;
    pub const SCAUSE_EXCP_LOAD_MISALIGNED: usize = 4;
    pub const SCAUSE_EXCP_LOAD_FAULT: usize = 5;
    pub const SCAUSE_EXCP_STORE_MISALIGNED: usize = 6;
    pub const SCAUSE_EXCP_STORE_FAULT: usize = 7;
    pub const SCAUSE_EXCP_ENV_CALL_FROM_USER: usize = 8;
    pub const SCAUSE_EXCP_ENV_CALL_FROM_SUPERV: usize = 9;
    pub const SCAUSE_EXCP_INSTRUCTION_PAGE_FAULT: usize = 12;
    pub const SCAUSE_EXCP_LOAD_PAGE_FAULT: usize = 13;
    pub const SCAUSE_EXCP_STORE_PAGE_FAULT: usize = 15;
    pub const SCAUSE_EXCP_SOFTWARE_CHECK: usize = 18;
    pub const SCAUSE_EXCP_HARDWARE_ERROR: usize = 19;
}

global_asm!(include_str!("./trap_entry.s"));

unsafe extern "C" {
    static _trap_entry: u8;
}

#[inline(always)]
fn get_scause() -> usize {
    unsafe {
        let scause: usize;

        asm!("csrr {scause}, scause", scause = out(reg) scause);

        scause
    }
}

#[unsafe(no_mangle)]
extern "C" fn rust_trap() -> ! {
    let scause = get_scause();

    let is_interrupt = scause & consts::SCAUSE_INTERRUPT_MASK > 0;

    if !is_interrupt {
        panic!("unexpected exception occurred, scause: {scause:x}");
    }

    let code = scause & consts::SCAUSE_CODE_MASK;

    let itype = match code {
        consts::SCAUSE_INTR_SUPERVISOR_TIMER => InterruptType::Timer,
        _ => panic!("Unexpected interrupt occurred, scause: {scause:x}"),
    };

    crate::interrupts::interrupt_handler(itype);
}

pub fn init() {
    unsafe {
        asm!("csrw stvec, {handler}", handler = in(reg) &_trap_entry);
        asm!("csrw sie, {val}", val = in(reg) consts::SIE_STIE);
    }
}

pub use super::sbi::schedule_timer;
