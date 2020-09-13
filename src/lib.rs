#[macro_use]
extern crate lazy_static;

#[path = "modbus-context.rs"]
pub mod context;

include!("frame.rs");
