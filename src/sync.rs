use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

use crate::smp;

pub struct SpinLock<T>(AtomicBool, UnsafeCell<T>);

unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}

pub struct SpinLockGuard<'a, T: 'a>(&'a SpinLock<T>);

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0.1.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.1.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.0.0.store(false, Ordering::Release);
    }
}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> SpinLock<T> {
        SpinLock(AtomicBool::new(false), UnsafeCell::new(value))
    }

    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        while self.0.swap(true, Ordering::Acquire) {}

        SpinLockGuard(self)
    }
}

// while acquired, the current thread can't preempt.
pub struct Critical<T>(UnsafeCell<T>);

unsafe impl<T: Send> Send for Critical<T> {}
unsafe impl<T: Send> Sync for Critical<T> {}

pub struct CriticalGuard<'a, T: 'a>(&'a Critical<T>);

impl<T> Deref for CriticalGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0.0.get() }
    }
}

impl<T> DerefMut for CriticalGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.0.get() }
    }
}

impl<T> Drop for CriticalGuard<'_, T> {
    fn drop(&mut self) {
        smp::get_ctx().preempt_count.fetch_sub(1, Ordering::Release);
    }
}

impl<T> Critical<T> {
    pub const fn new(value: T) -> Critical<T> {
        Critical(UnsafeCell::new(value))
    }

    // unsafe; it's the owner's job to make sure the data isn't shared between
    // cores.
    pub unsafe fn lock(&self) -> CriticalGuard<'_, T> {
        smp::get_ctx().preempt_count.fetch_add(1, Ordering::Acquire);

        CriticalGuard(self)
    }
}
