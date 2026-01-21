use core::arch::global_asm;

global_asm!(include_str!("entry.s"));

#[unsafe(no_mangle)]
extern "C" fn rust_entry(_hart_id: u64, _fdt_addr: u64) -> ! {
    crate::main();
}
