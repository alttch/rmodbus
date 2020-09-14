/// Default error
#[derive(Debug)]
pub enum ErrorKind {
    ContextOOB,
    OOB,
}

pub trait VectorTrait<T: Copy> {
    fn add(&mut self, value: T) -> Result<(), ErrorKind>;
    fn add_bulk(&mut self, other: &[T]) -> Result<(), ErrorKind>;
}

#[macro_use]
extern crate lazy_static;

#[path = "server.rs"]
pub mod server;
