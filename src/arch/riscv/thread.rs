use core::{arch::global_asm, mem};

use alloc::boxed::Box;

global_asm!(include_str!("./thread_switch.s"));

const THREAD_STACK_SIZE: usize = 8 * 1024_usize.pow(2);

#[repr(align(16))]
#[repr(C)]
struct ThreadStack([u8; THREAD_STACK_SIZE]);

const _: () = assert!(mem::offset_of!(Thread, registers) == 0);
const _: () = assert!(mem::offset_of!(Thread, pc) == 248);

#[repr(C)]
pub struct Thread {
    // skips x0; registers[0] is x1 and so on
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

impl Thread {
    pub fn spawn(f: impl FnOnce() + Send + 'static) -> Thread {
        let func_boxed: Box<dyn FnOnce()> = Box::new(f);
        let func_boxed_boxed = Box::new(func_boxed);

        let stack = unsafe {
            let mut boxed = Box::<ThreadStack>::new_uninit();
            boxed.as_mut_ptr().write_bytes(0, 1);
            Box::into_raw(boxed.assume_init())
        };

        let registers = [0; 31];

        let stack_top = stack as usize + THREAD_STACK_SIZE;

        let mut thread = Thread {
            stack,
            registers,
            pc: thread_entry as *const () as usize,
        };

        thread.registers[1] = stack_top; // x2 (sp)
        thread.registers[9] = Box::into_raw(func_boxed_boxed) as usize; // x10 (a0)

        thread
    }

    pub fn switch_to_thread(&mut self) -> ! {
        unsafe {
            thread_switch(&raw const *self);
        }
    }
}

unsafe extern "C" {
    fn thread_switch(ctx: *const Thread) -> !;
}
