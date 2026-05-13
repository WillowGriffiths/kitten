use core::{
    mem, ptr,
    sync::atomic::{AtomicPtr, AtomicUsize},
};

use alloc::{boxed::Box, vec::Vec};

use crate::{
    DeviceTreeInfo,
    arch::{self, boot::BootInfo, pagetable},
    scheduler::{Scheduler, Task},
};

const STACK_SIZE: usize = 8 * 1024_usize.pow(2);

#[repr(C)]
#[repr(align(16))]
pub struct CpuStack([u8; STACK_SIZE]);

const _: () = assert!(mem::offset_of!(BootRes, stack_top) == 0);
const _: () = assert!(mem::offset_of!(BootRes, pagetable) == 8);

#[repr(C)]
pub struct BootRes {
    pub stack_top: *const u8,
    pub pagetable: usize,
    pub cpu_id: usize,
    pub stack: Box<CpuStack>,
    pub device_tree_info: DeviceTreeInfo,
}

unsafe impl Send for BootRes {}

pub struct CpuCtx {
    pub cpu_id: usize,
    pub current_task: AtomicPtr<Task>,
    pub scheduler: Scheduler,
    pub preempt_count: AtomicUsize,
}

const _: () = {
    const fn is_sync<T: Sync>() {}
    is_sync::<CpuCtx>();
};

pub fn create_ctx(cpu_id: usize, device_tree_info: DeviceTreeInfo) {
    let ctx = Box::leak(Box::new(CpuCtx {
        cpu_id,
        current_task: AtomicPtr::new(ptr::null_mut()),
        scheduler: Scheduler::new(device_tree_info.timebase_frequency)
            .expect("Failed to create scheduler"),
        preempt_count: 0.into(),
    }));

    arch::store_ctx(ctx);
}

pub fn get_ctx() -> &'static CpuCtx {
    arch::get_ctx()
}

pub fn init(boot_info: &BootInfo, device_tree_info: DeviceTreeInfo) {
    let cpus = (0..boot_info.cpus - 1)
        .map(|i| {
            let index = if i < boot_info.boot_cpu as usize {
                i
            } else {
                i + 1
            };

            let stack = unsafe {
                let mut boxed = Box::<CpuStack>::new_uninit();
                boxed.as_mut_ptr().write_bytes(0, 1);
                boxed.assume_init()
            };
            let stack_top = unsafe { (&raw const *stack as *const u8).byte_add(STACK_SIZE) };

            let res = Box::new(BootRes {
                stack_top,
                pagetable: pagetable::get_pagetable(),
                cpu_id: index,
                stack,
                device_tree_info,
            });

            (index, res)
        })
        .collect::<Vec<_>>();

    for (index, res) in cpus {
        arch::start_cpu(index, res);
    }
}
