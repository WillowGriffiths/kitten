use core::{
    alloc::Layout,
    cell::UnsafeCell,
    mem,
    ptr::{self, NonNull},
    sync::atomic::AtomicUsize,
};

use alloc::alloc::{AllocError, Allocator};
use alloc::{self, boxed::Box, vec::Vec};

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

const _: () = assert!(mem::offset_of!(CpuCtx, irq_stack_top) == 0);
const _: () = assert!(mem::offset_of!(CpuCtx, user_scratch) == 8);
const _: () = assert!(mem::offset_of!(CpuCtx, current_task) == 16);

// irq_stack_top: immutable.
// user_sp_scratch: IRQ-only; not atomic.
// current_task: IRQ-only; not atomic.
// last_time: IRQ-only; not atomic.
//
// cpu_id: immutable.
// scheduler: blocks preemption; not atomic.
// preempt_count: used as a memory barrier; atomic.
#[repr(C)]
pub struct CpuCtx {
    pub irq_stack_top: NonNull<u8>,
    pub user_scratch: UnsafeCell<usize>,
    pub current_task: UnsafeCell<*mut Task>,
    pub last_time: UnsafeCell<usize>,

    pub cpu_id: usize,
    pub scheduler: Scheduler,
    pub preempt_count: AtomicUsize,
}

unsafe impl Sync for CpuCtx {}

const _: () = {
    const fn is_sync<T: Sync>() {}
    is_sync::<CpuCtx>();
};

pub fn create_ctx(cpu_id: usize) -> Result<(), AllocError> {
    let irq_stack: NonNull<u8> = alloc::alloc::Global
        .allocate(Layout::new::<CpuStack>())?
        .cast();
    let irq_stack_top = unsafe { irq_stack.byte_add(STACK_SIZE) };

    let ctx: &mut CpuCtx = unsafe {
        alloc::alloc::Global
            .allocate(Layout::new::<CpuCtx>())?
            .cast()
            .as_mut()
    };

    *ctx = CpuCtx {
        irq_stack_top,
        user_scratch: UnsafeCell::new(0),
        current_task: UnsafeCell::new(ptr::null_mut()),
        last_time: UnsafeCell::new(arch::get_time()),

        cpu_id,
        scheduler: Scheduler::new()?,
        preempt_count: 0.into(),
    };

    arch::store_ctx(ctx);

    Ok(())
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
