use std::cell::UnsafeCell;

pub trait LockFreeData {}

pub struct LockFree<T: LockFreeData> {
    data: UnsafeCell<T>,
}

impl<T: LockFreeData> LockFree<T> {
    pub fn new(v: T) -> LockFree<T> {
        LockFree {
            data: UnsafeCell::new(v),
        }
    }

    pub fn get(&self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

unsafe impl<T: LockFreeData> Sync for LockFree<T> {}
unsafe impl<T: LockFreeData> Send for LockFree<T> {}
