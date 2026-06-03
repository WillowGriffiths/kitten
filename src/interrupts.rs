use core::sync::atomic::Ordering;

use crate::{arch, global_data, smp};

pub enum InterruptType {
    Timer,
}

fn timer_interrupt() {
    let now = arch::get_time();
    let last_time = unsafe { &mut *smp::get_ctx().last_time.get() };
    let ticks = now - *last_time;
    *last_time = now;

    let timebase = global_data().timebase_frequency;
    arch::interrupts::schedule_timer(now + timebase / TIMER_DIV);

    unsafe {
        (**smp::get_ctx().current_task.get()).tick(ticks);
    };
}

pub fn interrupt_handler(itype: InterruptType) -> ! {
    match itype {
        InterruptType::Timer => timer_interrupt(),
    }

    unsafe {
        if (**smp::get_ctx().current_task.get()).reschedule
            && smp::get_ctx().preempt_count.load(Ordering::Relaxed) == 0
        {
            smp::get_ctx().scheduler.schedule();
        }

        (**smp::get_ctx().current_task.get()).enter();
    }
}

// 1000 interrupts/second
const TIMER_DIV: usize = 1000;

pub fn init() {
    arch::interrupts::init();

    let now = arch::get_time();
    let timebase = crate::global_data().timebase_frequency;
    arch::interrupts::schedule_timer(now + timebase / TIMER_DIV);
}
