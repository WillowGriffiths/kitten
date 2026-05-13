use alloc::{boxed::Box, vec::Vec};

use crate::arch::{self, boot::BootInfo, pagetable};

const STACK_SIZE: usize = 8 * 1024_usize.pow(2);

#[repr(C)]
#[repr(align(16))]
pub struct CpuStack([u8; STACK_SIZE]);

#[repr(C)]
pub struct BootRes {
    pub stack: Option<Box<CpuStack>>,
    pub pagetable: usize,
    pub cpu_id: u64,
}

pub fn init(boot_info: &BootInfo) {
    let cpus = (0..(boot_info.cpus - 1) as u64)
        .map(|i| {
            let index = if i < boot_info.boot_cpu { i } else { i + 1 };

            let res = Box::new(BootRes {
                stack: unsafe { Some(Box::new_uninit().assume_init()) },
                pagetable: pagetable::get_pagetable(),
                cpu_id: index,
            });

            (index, res)
        })
        .collect::<Vec<_>>();

    for (index, res) in cpus {
        arch::start_cpu(index, res);
    }
}
