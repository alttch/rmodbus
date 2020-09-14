use fixedvec::FixedVec;
use spin::{Mutex, MutexGuard};

impl<'a, T: Copy> VectorTrait<T> for FixedVec<'a, T> {
    fn add(&mut self, value: T) -> Result<(), ErrorKind> {
        return match self.push(value) {
            Ok(_) => Ok(()),
            Err(_) => Err(ErrorKind::OOB),
        };
    }
    fn add_bulk(&mut self, values: &[T]) -> Result<(), ErrorKind> {
        return match self.push_all(values) {
            Ok(_) => Ok(()),
            Err(_) => Err(ErrorKind::OOB),
        };
    }
}

macro_rules! lock_mutex {
    ($v:path) => {
        $v.lock()
    }
}
