use core::{alloc::Layout, mem, ptr::NonNull};

use alloc::alloc::{AllocError, Allocator};

use crate::{
    arch::{self, thread::Thread},
    global_data, smp,
    sync::Critical,
};

const TIMESLICE_DIV: usize = 100;

struct SchedulerInner {
    tasks_head: NonNull<Task>,
}

const _: () = assert!(mem::offset_of!(Task, thread) == 0);

#[repr(C)]
pub struct Task {
    thread: Thread,

    skip: bool,

    next: NonNull<Task>,
    prev: NonNull<Task>,

    runtime: usize,
    pub reschedule: bool,
}

impl Task {
    pub fn tick(&mut self, ticks: usize) {
        self.runtime += ticks;

        if self.runtime > global_data().timebase_frequency / TIMESLICE_DIV {
            self.reschedule = true;
        }
    }

    pub fn enter(&mut self) -> ! {
        self.thread.switch_to_thread();
    }
}

impl SchedulerInner {
    fn new() -> Result<SchedulerInner, AllocError> {
        let mut sleep_task = Self::create_task(|| {
            loop {
                arch::wfi();
            }
        })?;

        unsafe {
            sleep_task.as_mut().skip = true;
        }

        Ok(SchedulerInner {
            tasks_head: sleep_task,
        })
    }

    fn create_task(f: impl FnMut() + Send + 'static) -> Result<NonNull<Task>, AllocError> {
        unsafe {
            let thread = Thread::spawn(f);

            let mut task: NonNull<Task> =
                alloc::alloc::Global.allocate(Layout::new::<Task>())?.cast();

            *task.as_mut() = Task {
                next: task,
                prev: task,

                skip: false,

                thread,
                reschedule: false,

                runtime: 0,
            };

            Ok(task)
        }
    }

    fn spawn(&mut self, f: impl FnMut() + Send + 'static) -> Result<(), AllocError> {
        unsafe {
            let mut task = Self::create_task(f)?;

            let mut last_task = self.tasks_head.as_ref().prev;
            last_task.as_mut().next = task;

            task.as_mut().next = self.tasks_head;
            task.as_mut().prev = last_task;

            self.tasks_head = task;

            Ok(())
        }
    }

    fn schedule(&mut self) {
        unsafe {
            let mut task = self.tasks_head;

            *smp::get_ctx().current_task.get() = task.as_ptr();

            self.tasks_head = task.as_mut().next;
            if self.tasks_head.as_mut().skip {
                self.tasks_head = self.tasks_head.as_mut().next;
            }

            task.as_mut().runtime = 0;
            task.as_mut().reschedule = false;
        }
    }
}

pub struct Scheduler(Critical<SchedulerInner>);

// Sync is implemented as the scheduler is shared between threads.
// It must not be shared between cores.
unsafe impl Sync for Scheduler {}

impl Scheduler {
    pub fn new() -> Result<Scheduler, AllocError> {
        Ok(Scheduler(Critical::new(SchedulerInner::new()?)))
    }

    pub fn spawn(&self, f: impl FnMut() + Send + 'static) -> Result<(), AllocError> {
        unsafe { self.0.lock() }.spawn(f)
    }

    pub fn schedule(&self) {
        unsafe { self.0.lock() }.schedule();
    }
}

pub fn sleep(ms: usize) {
    let ticks = global_data().timebase_frequency * ms / 1000;
    let start = arch::get_time();
    let end = start + ticks;

    loop {
        let now = arch::get_time();
        if now >= end {
            return;
        }
    }
}
