use core::{alloc::Layout, ptr::NonNull};

use alloc::alloc::{AllocError, Allocator};

use crate::{
    arch::{self, thread::Thread},
    sync::Critical,
};

struct SchedulerInner {
    timebase_frequency: usize,

    tasks_head: NonNull<Task>,
}

pub struct Task {
    next: NonNull<Task>,
    prev: NonNull<Task>,

    thread: Thread,
    reschedule: bool,
}

impl SchedulerInner {
    fn new(timebase_frequency: usize) -> Result<SchedulerInner, AllocError> {
        let sleep_task = Self::create_task(|| {
            loop {
                arch::wfi();
            }
        })?;

        Ok(SchedulerInner {
            timebase_frequency,
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

                thread,
                reschedule: false,
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

    fn schedule(&mut self) -> ! {
        unsafe {
            self.tasks_head.as_mut().thread.switch_to_thread();
        }
    }
}

pub struct Scheduler(Critical<SchedulerInner>);

// Sync is implemented as the scheduler is shared between threads.
// It must not be shared between cores.
unsafe impl Sync for Scheduler {}

impl Scheduler {
    pub fn new(timebase_frequency: usize) -> Result<Scheduler, AllocError> {
        Ok(Scheduler(Critical::new(SchedulerInner::new(
            timebase_frequency,
        )?)))
    }

    pub fn spawn(&self, f: impl FnMut() + Send + 'static) -> Result<(), AllocError> {
        unsafe { self.0.lock() }.spawn(f)
    }

    pub fn schedule(&self) -> ! {
        unsafe { self.0.lock() }.schedule();
    }
}
