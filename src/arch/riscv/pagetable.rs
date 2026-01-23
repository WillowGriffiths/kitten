use core::arch::asm;

use crate::memory::{self, MemoryInfo};

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
        let addr = memory::to_phys(pagetable as u64);
        let satp = (8 << 60) | (addr >> 12);

        asm!(
            "sfence.vma zero,zero",
            "csrw satp, {satp}",
            "sfence.vma zero,zero",
            satp = in(reg) satp,
        );
    }
}

pub(super) fn setup(memory_info: &MemoryInfo) {
    unsafe {
        let kernel_page_l2 = (memory_info.kernel.virt >> 30) & 0x1ff;
        let kernel_physical = memory_info.kernel.phys >> 30 << 30;

        // TODO: map kernel with more granularity of permission bits
        ROOT_PAGETABLE.0[kernel_page_l2 as usize] =
            make_pte(kernel_physical, PTE_V | PTE_R | PTE_W | PTE_X);

        let mut mapping_len = 0;
        while mapping_len < memory_info.memory.len {
            let page_physical = memory_info.memory.phys + mapping_len;
            let page_virtual = memory_info.memory.virt + mapping_len;
            let l2 = (page_virtual >> 30) & 0x1ff;

            ROOT_PAGETABLE.0[l2 as usize] = make_pte(page_physical, PTE_V | PTE_R | PTE_W | PTE_X);

            mapping_len += 512 * 512 * 4096;
        }

        set_pagetable(&raw const ROOT_PAGETABLE);
    }
}
