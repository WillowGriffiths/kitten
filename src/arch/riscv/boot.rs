use core::arch::global_asm;

global_asm!(include_str!("entry.s"));

#[unsafe(no_mangle)]
extern "C" fn rust_entry(hart_id: u64, fdt: *const u32) -> ! {
    crate::arch::println!("We are running on hart {hart_id}");
    super::device_tree::parse(fdt);
    crate::main();
}
