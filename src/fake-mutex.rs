use core::cell::UnsafeCell;
use core::default::Default;
use core::marker::Sync;
use core::ops::{Deref, DerefMut};

pub struct Mutex<T: ?Sized> {
    data: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T: ?Sized + 'a> {
    data: &'a mut T,
}

unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(user_data: T) -> Mutex<T> {
        Mutex {
            data: UnsafeCell::new(user_data),
        }
    }

    pub fn into_inner(self) -> T {
        let Mutex { data, .. } = self;
        data.into_inner()
    }
}

impl<T: ?Sized> Mutex<T> {
    pub fn lock(&self) -> MutexGuard<T> {
        MutexGuard {
            data: unsafe { &mut *self.data.get() },
        }
    }
}

impl<T: ?Sized + Default> Default for Mutex<T> {
    fn default() -> Mutex<T> {
        Mutex::new(Default::default())
    }
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        &*self.data
    }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        &mut *self.data
    }
}
