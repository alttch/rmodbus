use crate::ErrorKind;

#[allow(clippy::module_name_repetitions)]
pub trait VectorTrait<T: Copy> {
    fn push(&mut self, value: T) -> Result<(), ErrorKind>;
    fn extend(&mut self, other: &[T]) -> Result<(), ErrorKind>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn clear(&mut self);
    fn cut_end(&mut self, len_to_cut: usize, value: T);
    fn as_slice(&self) -> &[T];
    fn as_mut_slice(&mut self) -> &mut [T];
    fn resize(&mut self, new_len: usize, value: T);
    fn replace(&mut self, index: usize, value: T);
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;
#[cfg(all(feature = "std", not(feature = "alloc")))]
use std::vec::Vec;

#[cfg(any(feature = "alloc", feature = "std"))]
impl<T: Copy> VectorTrait<T> for Vec<T> {
    #[inline]
    fn push(&mut self, value: T) -> Result<(), ErrorKind> {
        Vec::push(self, value);
        Ok(())
    }
    #[inline]
    fn extend(&mut self, values: &[T]) -> Result<(), ErrorKind> {
        Vec::extend_from_slice(self, values);
        Ok(())
    }
    #[inline]
    fn len(&self) -> usize {
        Vec::len(self)
    }
    #[inline]
    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }
    #[inline]
    fn clear(&mut self) {
        Vec::clear(self);
    }
    fn cut_end(&mut self, len_to_cut: usize, value: T) {
        let len = self.len();
        if len_to_cut >= len {
            self.clear();
        } else {
            self.resize(len - len_to_cut, value);
        }
    }
    #[inline]
    fn as_slice(&self) -> &[T] {
        Vec::as_slice(self)
    }
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [T] { Vec::as_mut_slice(self) }
    #[inline]
    fn resize(&mut self, new_len: usize, value: T) { Vec::resize(self, new_len, value) }
    #[inline]
    fn replace(&mut self, index: usize, value: T) {
        self[index] = value;
    }
}

#[cfg(any(feature = "fixedvec", test))]
use fixedvec::FixedVec;

#[cfg(any(feature = "fixedvec", test))]
impl<'a, T: Copy> VectorTrait<T> for FixedVec<'a, T> {
    #[inline]
    fn push(&mut self, value: T) -> Result<(), ErrorKind> {
        FixedVec::push(self, value).map_err(|_| ErrorKind::OOB)
    }
    #[inline]
    fn extend(&mut self, values: &[T]) -> Result<(), ErrorKind> {
        FixedVec::push_all(self, values).map_err(|_| ErrorKind::OOB)
    }
    #[inline]
    fn len(&self) -> usize {
        FixedVec::len(self)
    }
    #[inline]
    fn is_empty(&self) -> bool {
        FixedVec::is_empty(self)
    }
    #[inline]
    fn clear(&mut self) {
        FixedVec::clear(self);
    }
    fn cut_end(&mut self, len_to_cut: usize, value: T) {
        let len = self.len();
        if len_to_cut >= len {
            self.clear();
        } else {
            self.resize(len - len_to_cut, value);
        }
    }
    #[inline]
    fn as_slice(&self) -> &[T] {
        FixedVec::as_slice(self)
    }
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [T] { FixedVec::as_mut_slice(self) }
    #[inline]
    fn resize(&mut self, new_len: usize, value: T) { FixedVec::resize(self, new_len, value) }
    #[inline]
    fn replace(&mut self, index: usize, value: T) {
        self[index] = value;
    }
}

#[cfg(feature = "heapless")]
use heapless::Vec as HeaplessVec;

#[cfg(feature = "heapless")]
impl<T: Copy, const N: usize> VectorTrait<T> for HeaplessVec<T, N> {
    #[inline]
    fn push(&mut self, value: T) -> Result<(), ErrorKind> {
        HeaplessVec::push(self, value).map_err(|_| ErrorKind::OOB)
    }
    #[inline]
    fn extend(&mut self, values: &[T]) -> Result<(), ErrorKind> {
        self.extend_from_slice(values).map_err(|_| ErrorKind::OOB)
    }
    #[inline]
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
    #[inline]
    fn is_empty(&self) -> bool {
        HeaplessVec::is_empty(self)
    }
    #[inline]
    fn clear(&mut self) {
        HeaplessVec::clear(self);
    }
    fn cut_end(&mut self, len_to_cut: usize, value: T) {
        let len = self.len();
        if len_to_cut >= len {
            self.clear();
        } else {
            self.resize(len - len_to_cut, value).unwrap();
        }
    }
    #[inline]
    fn as_slice(&self) -> &[T] {
        HeaplessVec::as_slice(self)
    }
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [T] { HeaplessVec::as_mut_slice(self) }
    #[inline]
    fn resize(&mut self, new_len: usize, value: T) { HeaplessVec::resize(self, new_len, value)? }
    #[inline]
    fn replace(&mut self, index: usize, value: T) {
        self[index] = value;
    }
}
