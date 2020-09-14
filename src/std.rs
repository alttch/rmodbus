use std::sync::{Mutex, MutexGuard};

impl<T: Copy> VectorTrait<T> for Vec<T> {
    fn add(&mut self, value: T) -> Result<(), ErrorKind> {
        self.push(value);
        return Ok(());
    }
    fn add_bulk(&mut self, values: &[T]) -> Result<(), ErrorKind> {
        self.extend_from_slice(values);
        return Ok(());
    }
}

macro_rules! lock_mutex {
    ($v:path) => {
        $v.lock().unwrap()
    }
}
