use alloc::{boxed::Box, vec::Vec};

use crate::arch::{self, boot::BootInfo, pagetable};

const STACK_SIZE: usize = 8 * 1024_usize.pow(2);

#[repr(C)]
#[repr(align(16))]
pub struct CpuStack([u8; STACK_SIZE]);

#[repr(C)]
pub struct BootRes {
    pub stack_top: *const u8,
    pub pagetable: usize,
    pub cpu_id: u64,
    pub stack: Box<CpuStack>,
}

pub fn init(boot_info: &BootInfo) {
    let cpus = (0..(boot_info.cpus - 1) as u64)
        .map(|i| {
            let index = if i < boot_info.boot_cpu { i } else { i + 1 };

            let stack = unsafe { Box::new_uninit().assume_init() };
            let stack_top = unsafe { (&raw const *stack as *const u8).byte_add(STACK_SIZE) };

            let res = Box::new(BootRes {
                stack_top,
                pagetable: pagetable::get_pagetable(),
                cpu_id: index,
                stack,
            });

            (index, res)
        })
        .collect::<Vec<_>>();

    for (index, res) in cpus {
        arch::start_cpu(index, res);
    }
}
