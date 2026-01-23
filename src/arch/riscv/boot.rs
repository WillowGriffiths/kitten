use crate::arch::println;
use core::arch::global_asm;

global_asm!(include_str!("entry.s"));

#[unsafe(no_mangle)]
extern "C" fn rust_entry(hart_id: u64, fdt: *const u8, kernel_start: u64, kernel_end: u64) -> ! {
    crate::arch::println!("We are running on hart {hart_id}");
    let fdt = unsafe { fdt.sub(0x80000000).add(0xffffffff80000000) };
    let fdt_info = super::device_tree::FdtInfo::new(fdt);
    let boot_info = fdt_info.boot_info(kernel_start, kernel_end);

    println!(
        "found {} harts, {}B of memory",
        boot_info.cpus, boot_info.memory.len
    );

    crate::main(boot_info);
}
