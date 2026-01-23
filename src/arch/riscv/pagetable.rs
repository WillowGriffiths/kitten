use core::arch::asm;

use crate::arch::boot::BootInfo;

#[repr(align(4096))]
#[repr(C)]
struct PageTable([u64; 512]);

static mut ROOT_PAGETABLE: PageTable = PageTable([0; 512]);

const PTE_V: u64 = 1 << 0; // valid
const PTE_R: u64 = 1 << 1; // readable
const PTE_W: u64 = 1 << 2; // writable
const PTE_X: u64 = 1 << 3; // executable
const PTE_U: u64 = 1 << 4; // user can access

fn make_pte(addr: u64, flags: u64) -> u64 {
    (addr >> 12) << 10 | flags
}

fn set_pagetable(pagetable: *const PageTable) {
    unsafe {
        let addr = pagetable as u64 - 0xffffffff80000000 + 0x80000000;
        let satp = (8 << 60) | (addr >> 12);

        asm!(
            "sfence.vma zero,zero",
            "csrw satp, {satp}",
            "sfence.vma zero,zero",
            satp = in(reg) satp,
        );
    }
}

pub(super) fn setup(boot_info: &BootInfo) {
    let kernel_page_l2 = (boot_info.kernel_virtual.start >> 30) & 0x1ff;
    let kernel_physical = boot_info.kernel_memory.start >> 30 << 30;

    unsafe {
        ROOT_PAGETABLE.0[kernel_page_l2 as usize] =
            make_pte(kernel_physical, PTE_V | PTE_R | PTE_W | PTE_X);

        set_pagetable(&raw const ROOT_PAGETABLE);
    }
}
