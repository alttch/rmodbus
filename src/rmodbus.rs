/// Default error
#[derive(Debug)]
pub enum ErrorKind {
    OOB,
    OOBContext,
    FrameBroken,
    FrameCRCError
}

pub trait VectorTrait<T: Copy> {
    fn add(&mut self, value: T) -> Result<(), ErrorKind>;
    fn add_bulk(&mut self, other: &[T]) -> Result<(), ErrorKind>;
    fn get_len(&self) -> usize;
    fn clear_all(&mut self);
    fn cut_end(&mut self, len_to_cut: usize, value: T);
    fn get_slice(&self) -> &[T];
}

#[path = "server.rs"]
pub mod server;
