use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

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
