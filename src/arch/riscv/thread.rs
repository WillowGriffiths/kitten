use core::arch::global_asm;

use alloc::boxed::Box;

global_asm!(include_str!("./thread_switch.s"));

const THREAD_STACK_SIZE: usize = 8 * 1024_usize.pow(2);

#[repr(align(16))]
#[repr(C)]
struct ThreadStack([u8; THREAD_STACK_SIZE]);

#[repr(C)]
pub struct ThreadContext {
    registers: [usize; 31],
    pc: usize,
    stack: *mut ThreadStack,
}

extern "C" fn thread_entry(func: Box<Box<dyn FnOnce()>>) -> ! {
    func();

    loop {
        crate::arch::wfi();
    }
}

pub fn new_thread(func: impl FnOnce() + Send + 'static) -> Box<ThreadContext> {
    let func_boxed: Box<dyn FnOnce()> = Box::new(func);
    let func_boxed_boxed = Box::new(func_boxed);

    let stack = Box::into_raw(unsafe { Box::new_zeroed().assume_init() });
    let registers = [0; 31];

    let stack_top = stack as usize + THREAD_STACK_SIZE;

    let mut ctx = Box::new(ThreadContext {
        stack,
        registers,
        pc: thread_entry as *const () as usize,
    });

    ctx.registers[1] = stack_top; // x2 (sp)
    ctx.registers[3] = &raw mut *ctx as usize; // x3 (tp)
    ctx.registers[9] = Box::into_raw(func_boxed_boxed) as usize; // x10 (a0)

    ctx
}

pub fn switch_to_thread(ctx: Box<ThreadContext>) -> ! {
    let ptr = Box::into_raw(ctx);

    unsafe {
        thread_switch(ptr);
    }
}

unsafe extern "C" {
    fn thread_switch(ctx: *const ThreadContext) -> !;
}
