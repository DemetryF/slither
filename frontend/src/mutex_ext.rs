use std::sync::Mutex;

pub trait MutexExt<T> {
    fn lock_with<R>(&self, f: impl FnOnce(&T) -> R) -> R;
    fn lock_with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn lock_with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.lock().unwrap())
    }

    fn lock_with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        f(&mut self.lock().unwrap())
    }
}
