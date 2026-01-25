use core::alloc::GlobalAlloc;

use crate::arch::{boot::BootInfo, println};

struct BuddyAllocator;

unsafe impl GlobalAlloc for BuddyAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        todo!()
    }
}

#[global_allocator]
static ALLOCATOR: BuddyAllocator = BuddyAllocator;

pub fn setup(boot_info: &BootInfo) {
    let mut free_start = boot_info.memory_info.kernel.phys + boot_info.memory_info.kernel.len;

    for resv in &boot_info.resv[0..boot_info.resv_count] {
        println!("{resv:#x?}");
        free_start = free_start.max(resv.start + resv.len);
    }

    let page_size = 4096;
    if !free_start.is_multiple_of(page_size) {
        free_start = (free_start / page_size + 1) * page_size;
    }

    let memory_end = boot_info.memory_info.memory.phys + boot_info.memory_info.memory.len;
    let free_size = memory_end - free_start;
    let memory_size = boot_info.memory_info.memory.len;

    println!("{free_size}B of free memory from {free_start:#x} out of {memory_size}B total");
}
