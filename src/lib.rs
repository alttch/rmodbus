//! # rmodbus - Modbus for Rust
//! 
//! A framework to build fast and reliable Modbus-powered applications.
//! 
//! Cargo crate: https://crates.io/crates/rmodbus
//! 
//! ## What is rmodbus
//! 
//! rmodbus is not a yet another Modbus server. rmodbus is a set of tools to
//! quickly build Modbus-powered applications.
//! 
//! ## Why yet another Modbus lib?
//! 
//! * rmodbus is transport and protocol independent
//! 
//! * rmodbus is platform independent (**no\_std is fully supported!**)
//! 
//! * can be easily used in blocking and async (non-blocking) applications
//! 
//! * tuned for speed and reliability
//! 
//! * provides a set of tools to easily work with Modbus context
//! 
//! * supports server frame processing for Modbus TCP/UDP, RTU and ASCII.
//! 
//! * server context can be easily, managed, imported and exported
//! 
//! ## So the server isn't included?
//! 
//! Yes, there's no server included. You build the server by your own. You choose
//! protocol, technology and everything else. rmodbus just process frames and works
//! with Modbus context.
//! 
//! Here's an example of a simple TCP blocking server:
//! 
//! ```rust,ignore
//! use std::io::{Read, Write};
//! use std::net::TcpListener;
//! use std::thread;
//! 
//! use rmodbus::server::{ModbusFrame, ModbusProto, process_frame};
//! 
//! pub fn tcpserver(unit: u8, listen: &str) {
//!     let listener = TcpListener::bind(listen).unwrap();
//!     println!("listening started, ready to accept");
//!     for stream in listener.incoming() {
//!         thread::spawn(move || {
//!             println!("client connected");
//!             let mut stream = stream.unwrap();
//!             loop {
//!                 let mut buf: ModbusFrame = [0; 256];
//!                 let mut response = Vec::new(); // for nostd use FixedVec with alloc [u8;256]
//!                 if stream.read(&mut buf).unwrap_or(0) == 0 {
//!                     return;
//!                 }
//!                 if process_frame(unit, &buf, ModbusProto::TcpUdp, &mut response).is_err() {
//!                         println!("server error");
//!                         return;
//!                     }
//!                 println!("{:x?}", response.as_slice());
//!                 if !response.is_empty() {
//!                     if stream.write(response.as_slice()).is_err() {
//!                         return;
//!                     }
//!                 }
//!             }
//!         });
//!     }
//! }
//! ```
//! 
//! There are also examples for Serial-RTU, Serial-ASCII and UDP in *examples*
//! folder (if you're reading this text somewhere else, visit [rmodbus project
//! repository](https://github.com/alttch/rmodbus).
//! 
//! Running examples:
//! 
//! ```shell
//! cargo run --example app --features=std
//! cargo run --example tcpserver --features=std
//! ```
//! 
//! ## Modbus context
//! 
//! The rule is simple: one standard Modbus context per application. 10k+10k 16-bit
//! registers and 10k+10k coils are usually more than enough. This takes about
//! 43Kbytes of RAM, but if you need to reduce context size, download library
//! source and change *CONTEXT_SIZE* constant in "context.rs".
//! 
//! rmodbus server context is thread-safe, easy to use and has a lot of functions.
//! 
//! The context is created automatically, as soon as the library is imported. No
//! additional action is required.
//! 
//! Every time Modbus context is accessed, a context mutex must be locked. This
//! slows down a performance, but guarantees that the context always has valid data
//! after bulk-sets or after 32-bit data types were written. So make sure your
//! application locks context only when required and only for a short period time.
//! 
//! There are two groups of context functions:
//! 
//! * High-level API: simple functions like *coil_get* automatically lock the
//!   context but do this every time when called. Use this for testing or if the
//!   speed is not important.
//! 
//! * Advanced way is to use low-level API, lock the context manually and then call
//!   proper functions, like *set*, *set_f32* etc.
//! 
//! Take a look at simple PLC example:
//! 
//! ```rust,ignore
//! use rmodbus::server::context;
//! use std::fs::File;
//! use std::io::prelude::*;
//! use std::sync::MutexGuard;
//! 
//! fn looping() {
//!     loop {
//!         // READ WORK MODES ETC
//!         let mut ctx = context::CONTEXT.lock().unwrap();
//!         let _param1 = context::get(1000, &ctx.holdings).unwrap();
//!         let _param2 = context::get_f32(1100, &ctx.holdings).unwrap(); // ieee754 f32
//!         let _param3 = context::get_u32(1200, &ctx.holdings).unwrap(); // u32
//!         let cmd = context::get(1500, &ctx.holdings).unwrap();
//!         context::set(1500, 0, &mut ctx.holdings).unwrap();
//!         if cmd != 0 {
//!             println!("got command code {}", cmd);
//!             match cmd {
//!                 1 => {
//!                     println!("saving memory context");
//!                     let _ = save("/tmp/plc1.dat", &mut ctx).map_err(|_| {
//!                         eprintln!("unable to save context!");
//!                     });
//!                 }
//!                 _ => println!("command not implemented"),
//!             }
//!         }
//!         drop(ctx);
//!         // ==============================================
//!         // DO SOME JOB
//!         // ..........
//!         // WRITE RESULTS
//!         let mut ctx = context::CONTEXT.lock().unwrap();
//!         context::set(0, true, &mut ctx.coils).unwrap();
//!         context::set_bulk(10, &(vec![10, 20]), &mut ctx.holdings).unwrap();
//!         context::set_f32(20, 935.77, &mut ctx.inputs).unwrap();
//!     }
//! }
//! 
//! fn save(fname: &str, ctx: &MutexGuard<context::ModbusContext>) -> Result<(), std::io::Error> {
//!     let mut file = match File::create(fname) {
//!         Ok(v) => v,
//!         Err(e) => return Err(e),
//!     };
//!     for i in context::context_iter(&ctx) {
//!         match file.write(&[i]) {
//!             Ok(_) => {}
//!             Err(e) => return Err(e),
//!         }
//!     }
//!     match file.sync_all() {
//!         Ok(_) => {}
//!         Err(e) => return Err(e),
//!     }
//!     return Ok(());
//! }
//! ```
//! 
//! To let the above program communicate with outer world, Modbus server must be up
//! and running in the separate thread, asynchronously or whatever is preferred.
//! 
//! ## no\_std
//! 
//! rmodbus supports no\_std mode. Most of the library code is written the way to
//! support both std and no\_std.
//! 
//! Set dependency as:
//! 
//! ```toml
//! rmodbus = { version = "*", features = ["nostd"] }
//! ```
//! 
//! ## Small context
//! 
//! Default Modbus context has 10000 registers of each type, which requires 42500
//! bytes total. For systems with small RAM amount it's possible to reduce the
//! context size to the 1000 registers of each type (4250 bytes) with the following
//! feature:
//! 
//! ```toml
//! rmodbus = { version = "*", features = ["nostd", "smallcontext"] }
//! ```
//! 
//! ## Differences from 0.3.x
//! 
//! Starting from version 0.4:
//! 
//! * Modbus context is no longer created automatically and no mutex guard is
//!   provided by default. Use ModbusContext::new() to create context object and
//!   then use it as you wish - protect with any kind of Mutex, with RwLock or just
//!   put into UnsafeCell.
//! 
//! * Context SDK changes: all functions moved inside context, removed unnecessary
//!   ones, function args optimized.
//! 
//! ## Modbus client
//! 
//! Planned.
#![cfg_attr(feature = "nostd", no_std)]

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
#[macro_use]
extern crate fixedvec;

#[cfg(not(feature = "nostd"))]
impl<T: Copy> VectorTrait<T> for Vec<T> {
    fn add(&mut self, value: T) -> Result<(), ErrorKind> {
        self.push(value);
        return Ok(());
    }
    fn add_bulk(&mut self, values: &[T]) -> Result<(), ErrorKind> {
        self.extend_from_slice(values);
        return Ok(());
    }
    fn get_len(&self) -> usize {
        return self.len();
    }
    fn clear_all(&mut self) {
        self.clear();
    }
    fn cut_end(&mut self, len_to_cut: usize, value: T) {
        let len = self.len();
        if len_to_cut >= len {
            self.clear();
        } else {
            self.resize(len - len_to_cut, value);
        }
    }
    fn get_slice(&self) -> &[T] {
        return self.as_slice();
    }
}

use fixedvec::FixedVec;

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
    fn get_len(&self) -> usize {
        return self.len();
    }
    fn clear_all(&mut self) {
        self.clear();
    }
    fn cut_end(&mut self, len_to_cut: usize, value: T) {
        let len = self.len();
        if len_to_cut >= len {
            self.clear();
        } else {
            self.resize(len - len_to_cut, value);
        }
    }
    fn get_slice(&self) -> &[T] {
        return self.as_slice();
    }
}

include!("rmodbus.rs");

#[cfg(test)]
#[cfg(not(feature = "nostd"))]
mod tests {
    use super::server::context::*;
    use super::server::*;
    use super::ErrorKind;

    use crc16::*;
    use rand::Rng;

    use std::sync::RwLock;

    lazy_static! {
        pub static ref CTX: RwLock<ModbusContext> = RwLock::new(ModbusContext::new());
    }

    #[test]
    fn test_std_read_coils_as_bytes_oob() {
        let mut ctx = CTX.write().unwrap();
        let mut result = Vec::new();
        match ctx.get_coils_bulk(0, CONTEXT_SIZE as u16 + 1, &mut result) {
            Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
            Err(_) => assert!(true),
        }
        match ctx.get_coils_bulk(CONTEXT_SIZE as u16, 1, &mut result) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        ctx.get_coils_bulk((CONTEXT_SIZE - 1) as u16, 1, &mut result)
            .unwrap();
        match ctx.get_coils_bulk(CONTEXT_SIZE as u16 - 1, 2, &mut result) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        ctx.get_coil((CONTEXT_SIZE - 1) as u16).unwrap();
        match ctx.get_coil(CONTEXT_SIZE as u16) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        ctx.set_coil((CONTEXT_SIZE - 1) as u16, true).unwrap();
        match ctx.set_coil(CONTEXT_SIZE as u16, true) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_std_coil_get_set_bulk() {
        let mut ctx = CTX.write().unwrap();
        let mut data = Vec::new();
        let mut result = Vec::new();
        data.extend_from_slice(&[true; 2]);
        ctx.set_coils_bulk(5, &data.as_slice()).unwrap();
        ctx.get_coils_bulk(5, 2, &mut result).unwrap();
        assert_eq!(result, data);

        data.clear();
        result.clear();

        data.extend_from_slice(&[true; 18]);
        ctx.set_coils_bulk(25, &data.as_slice()).unwrap();
        ctx.get_coils_bulk(25, 18, &mut result).unwrap();
        assert_eq!(result, data);

        ctx.set_coil(28, true).unwrap();
        assert_eq!(ctx.get_coil(28).unwrap(), true);
        ctx.set_coil(28, false).unwrap();
        assert_eq!(ctx.get_coil(28).unwrap(), false);
    }

    #[test]
    fn test_std_read_discretes_as_bytes_oob() {
        let mut ctx = CTX.write().unwrap();
        let mut result = Vec::new();
        match ctx.get_discretes_bulk(0, CONTEXT_SIZE as u16 + 1, &mut result) {
            Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
            Err(_) => assert!(true),
        }
        match ctx.get_discretes_bulk(CONTEXT_SIZE as u16, 1, &mut result) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        ctx.get_discretes_bulk((CONTEXT_SIZE - 1) as u16, 1, &mut result)
            .unwrap();
        match ctx.get_discretes_bulk(CONTEXT_SIZE as u16 - 1, 2, &mut result) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        ctx.get_discrete((CONTEXT_SIZE - 1) as u16).unwrap();
        match ctx.get_discrete(CONTEXT_SIZE as u16) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        ctx.set_discrete((CONTEXT_SIZE - 1) as u16, true).unwrap();
        match ctx.set_discrete(CONTEXT_SIZE as u16, true) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_std_discrete_get_set_bulk() {
        let mut ctx = CTX.write().unwrap();
        let mut data = Vec::new();
        let mut result = Vec::new();
        data.extend_from_slice(&[true; 2]);
        ctx.set_discretes_bulk(5, &data.as_slice()).unwrap();
        ctx.get_discretes_bulk(5, 2, &mut result).unwrap();
        assert_eq!(result, data);

        data.clear();
        result.clear();

        data.extend_from_slice(&[true; 18]);
        ctx.set_discretes_bulk(25, &data.as_slice()).unwrap();
        ctx.get_discretes_bulk(25, 18, &mut result).unwrap();
        assert_eq!(result, data);

        ctx.set_discrete(28, true).unwrap();
        assert_eq!(ctx.get_discrete(28).unwrap(), true);
        ctx.set_discrete(28, false).unwrap();
        assert_eq!(ctx.get_discrete(28).unwrap(), false);
    }

    #[test]
    fn test_std_read_inputs_as_bytes_oob() {
        let mut ctx = CTX.write().unwrap();
        let mut result = Vec::new();
        match ctx.get_inputs_bulk(0, CONTEXT_SIZE as u16 + 1, &mut result) {
            Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
            Err(_) => assert!(true),
        }
        match ctx.get_inputs_bulk(CONTEXT_SIZE as u16, 1, &mut result) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        ctx.get_inputs_bulk((CONTEXT_SIZE - 1) as u16, 1, &mut result)
            .unwrap();
        match ctx.get_inputs_bulk(CONTEXT_SIZE as u16 - 1, 2, &mut result) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        ctx.get_input((CONTEXT_SIZE - 1) as u16).unwrap();
        match ctx.get_input(CONTEXT_SIZE as u16) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        ctx.set_input((CONTEXT_SIZE - 1) as u16, 0x55).unwrap();
        match ctx.set_input(CONTEXT_SIZE as u16, 0x55) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        match ctx.set_inputs_from_u32((CONTEXT_SIZE - 1) as u16, 0x55) {
            Ok(_) => assert!(false, "oob failed MAX u32"),
            Err(_) => assert!(true),
        }
        ctx.set_inputs_from_u32((CONTEXT_SIZE - 2) as u16, 0x9999)
            .unwrap();
        match ctx.set_inputs_from_u64((CONTEXT_SIZE - 3) as u16, 0x55) {
            Ok(_) => assert!(false, "oob failed MAX u64"),
            Err(_) => assert!(true),
        }
        ctx.set_inputs_from_u64((CONTEXT_SIZE - 4) as u16, 0x9999)
            .unwrap();
    }

    #[test]
    fn test_std_get_set_inputs() {
        let mut ctx = CTX.write().unwrap();
        let mut data = Vec::new();
        let mut result = Vec::new();

        ctx.clear_inputs();

        data.extend_from_slice(&[0x77; 2]);
        ctx.set_inputs_bulk(5, &data.as_slice()).unwrap();
        ctx.get_inputs_bulk(5, 2, &mut result).unwrap();
        assert_eq!(result, data);

        data.clear();
        result.clear();

        data.extend_from_slice(&[0x33; 18]);
        ctx.set_inputs_bulk(25, &data.as_slice()).unwrap();
        ctx.get_inputs_bulk(25, 18, &mut result).unwrap();
        assert_eq!(result, data);

        ctx.set_input(28, 99).unwrap();
        assert_eq!(ctx.get_input(28).unwrap(), 99);
        ctx.set_input(28, 95).unwrap();
        assert_eq!(ctx.get_input(28).unwrap(), 95);
        ctx.set_inputs_from_u32(100, 1234567).unwrap();
        assert_eq!(ctx.get_inputs_as_u32(100).unwrap(), 1234567);
        ctx.set_inputs_from_u64(90, 18_446_744_073_709_551_615)
            .unwrap();
        assert_eq!(
            ctx.get_inputs_as_u64(90).unwrap(),
            18_446_744_073_709_551_615
        );
        ctx.set_inputs_from_f32(200, 1234.567).unwrap();
        assert_eq!(ctx.get_inputs_as_f32(200).unwrap(), 1234.567);
    }

    #[test]
    fn test_std_read_holdings_as_bytes_oob() {
        let mut ctx = CTX.write().unwrap();
        let mut result = Vec::new();
        match ctx.get_holdings_bulk(0, CONTEXT_SIZE as u16 + 1, &mut result) {
            Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
            Err(_) => assert!(true),
        }
        match ctx.get_holdings_bulk(CONTEXT_SIZE as u16, 1, &mut result) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        ctx.get_holdings_bulk((CONTEXT_SIZE - 1) as u16, 1, &mut result)
            .unwrap();
        match ctx.get_holdings_bulk(CONTEXT_SIZE as u16 - 1, 2, &mut result) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        ctx.get_holding((CONTEXT_SIZE - 1) as u16).unwrap();
        match ctx.get_holding(CONTEXT_SIZE as u16) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        ctx.set_holding((CONTEXT_SIZE - 1) as u16, 0x55).unwrap();
        match ctx.set_holding(CONTEXT_SIZE as u16, 0x55) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        match ctx.set_holdings_from_u32((CONTEXT_SIZE - 1) as u16, 0x55) {
            Ok(_) => assert!(false, "oob failed MAX u32"),
            Err(_) => assert!(true),
        }
        ctx.set_holdings_from_u32((CONTEXT_SIZE - 2) as u16, 0x9999)
            .unwrap();
        match ctx.set_holdings_from_u64((CONTEXT_SIZE - 3) as u16, 0x55) {
            Ok(_) => assert!(false, "oob failed MAX u64"),
            Err(_) => assert!(true),
        }
        ctx.set_holdings_from_u64((CONTEXT_SIZE - 4) as u16, 0x9999)
            .unwrap();
    }

    #[test]
    fn test_std_get_set_holdings() {
        let mut ctx = CTX.write().unwrap();
        let mut data = Vec::new();
        let mut result = Vec::new();

        ctx.clear_holdings();

        data.extend_from_slice(&[0x77; 2]);
        ctx.set_holdings_bulk(5, &data.as_slice()).unwrap();
        ctx.get_holdings_bulk(5, 2, &mut result).unwrap();
        assert_eq!(result, data);

        data.clear();
        result.clear();

        data.extend_from_slice(&[0x33; 18]);
        ctx.set_holdings_bulk(25, &data.as_slice()).unwrap();
        ctx.get_holdings_bulk(25, 18, &mut result).unwrap();
        assert_eq!(result, data);

        ctx.set_holding(28, 99).unwrap();
        assert_eq!(ctx.get_holding(28).unwrap(), 99);
        ctx.set_holding(28, 95).unwrap();
        assert_eq!(ctx.get_holding(28).unwrap(), 95);
        ctx.set_holdings_from_u32(100, 1234567).unwrap();
        assert_eq!(ctx.get_holdings_as_u32(100).unwrap(), 1234567);
        ctx.set_holdings_from_u64(90, 18_446_744_073_709_551_615)
            .unwrap();
        assert_eq!(
            ctx.get_holdings_as_u64(90).unwrap(),
            18_446_744_073_709_551_615
        );
        ctx.set_holdings_from_f32(200, 1234.567).unwrap();
        assert_eq!(ctx.get_holdings_as_f32(200).unwrap(), 1234.567);
    }

    #[test]
    fn test_std_get_bools_as_u8() {
        let mut data = Vec::new();
        let mut ctx = CTX.write().unwrap();
        ctx.clear_coils();
        data.extend_from_slice(&[true, true, true, true, true, true, false, false]);
        ctx.set_coils_bulk(0, &data.as_slice()).unwrap();
        let mut result = Vec::new();
        ctx.get_coils_as_u8(0, 6, &mut result).unwrap();
        assert_eq!(*result.get(0).unwrap(), 0b00111111);
        result.clear();
        ctx.get_coils_as_u8(0, 5, &mut result).unwrap();
        assert_eq!(*result.get(0).unwrap(), 0b00011111);
        result.clear();

        data.clear();
        data.extend_from_slice(&[true, true, false, true, true, true, true, true]);
        ctx.set_coils_bulk(0, &data.as_slice()).unwrap();
        let mut result = Vec::new();
        ctx.get_coils_as_u8(0, 6, &mut result).unwrap();
        assert_eq!(*result.get(0).unwrap(), 0b00111011);
        result.clear();
        ctx.get_coils_as_u8(0, 5, &mut result).unwrap();
        assert_eq!(*result.get(0).unwrap(), 0b00011011);
        result.clear();

        data.clear();
        data.extend_from_slice(&[
            true, true, false, true, true, true, true, true, // byte 1
            true, true, true, true, false, false, true, false, // byte 2
            false, false, false, true, false, true, // byte 3
        ]);
        ctx.set_coils_bulk(0, &data.as_slice()).unwrap();
        let mut result = Vec::new();
        ctx.get_coils_as_u8(0, 22, &mut result).unwrap();
        assert_eq!(*result.get(0).unwrap(), 0b11111011);
        assert_eq!(*result.get(1).unwrap(), 0b01001111);
        assert_eq!(*result.get(2).unwrap(), 0b101000);
    }

    #[test]
    fn test_std_get_set_regs_as_u8() {
        let mut data = Vec::new();
        let mut ctx = CTX.write().unwrap();
        data.extend_from_slice(&[2, 45, 4559, 31, 394, 1, 9, 7, 0, 1, 9]);
        ctx.clear_holdings();
        ctx.set_holdings_bulk(0, &data.as_slice()).unwrap();
        let mut result = Vec::new();
        ctx.get_holdings_as_u8(0, data.len() as u16, &mut result)
            .unwrap();
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 2);
        for i in 0..10 {
            ctx.set_holding(i, 0).unwrap();
        }
        ctx.set_holdings_from_u8(0, &result.as_slice()).unwrap();
        let mut result = Vec::new();
        ctx.get_holdings_bulk(0, data.len() as u16, &mut result)
            .unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_std_get_set_bools_as_u8() {
        let mut ctx = CTX.write().unwrap();
        ctx.clear_coils();
        let mut data = Vec::new();
        data.extend_from_slice(&[
            true, true, true, false, true, true, true, true, true, false, false, false, false,
            false,
        ]);
        ctx.set_coils_bulk(0, &data.as_slice()).unwrap();
        ctx.set_coil(data.len() as u16, true).unwrap();
        ctx.set_coil(data.len() as u16 + 1, false).unwrap();
        ctx.set_coil(data.len() as u16 + 2, true).unwrap();
        let mut result = Vec::new();
        ctx.get_coils_as_u8(0, data.len() as u16, &mut result)
            .unwrap();
        ctx.set_coils_from_u8(0, data.len() as u16, &result.as_slice())
            .unwrap();
        let mut result = Vec::new();
        ctx.get_coils_bulk(0, data.len() as u16, &mut result)
            .unwrap();
        assert_eq!(result, data);
        result.clear();
        data.push(true);
        data.push(false);
        data.push(true);
        ctx.get_coils_bulk(0, data.len() as u16, &mut result)
            .unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_std_dump_restore() {
        let mut rng = rand::thread_rng();
        let mut mycoils: Vec<bool> = Vec::new();
        let mut mydiscretes: Vec<bool> = Vec::new();
        let mut myholdings: Vec<u16> = Vec::new();
        let mut myinputs: Vec<u16> = Vec::new();
        for _ in 0..CONTEXT_SIZE {
            mycoils.push(rng.gen());
            mydiscretes.push(rng.gen());
            myholdings.push(rng.gen());
            myinputs.push(rng.gen());
        }
        let mut ctx = CTX.write().unwrap();
        ctx.clear_all();
        ctx.set_coils_bulk(0, &mycoils).unwrap();
        ctx.set_discretes_bulk(0, &mydiscretes).unwrap();
        ctx.set_holdings_bulk(0, &myholdings).unwrap();
        ctx.set_inputs_bulk(0, &myinputs).unwrap();
        let mut dump: Vec<u8> = Vec::new();
        {
            for i in 0..CONTEXT_SIZE * 17 / 4 {
                dump.push(ctx.get_cell(i as u16).unwrap());
            }
        }
        ctx.clear_all();
        let mut offset = 0;
        for value in &dump {
            ctx.set_cell(offset, *value).unwrap();
            offset = offset + 1;
        }
        let mut result = Vec::new();
        ctx.get_coils_bulk(0, CONTEXT_SIZE as u16, &mut result)
            .unwrap();
        assert_eq!(result, mycoils);
        result.clear();
        ctx.get_discretes_bulk(0, CONTEXT_SIZE as u16, &mut result)
            .unwrap();
        assert_eq!(result, mydiscretes);
        result.clear();

        let mut result = Vec::new();
        ctx.get_inputs_bulk(0, CONTEXT_SIZE as u16, &mut result)
            .unwrap();
        assert_eq!(result, myinputs);
        result.clear();
        ctx.get_holdings_bulk(0, CONTEXT_SIZE as u16, &mut result)
            .unwrap();
        assert_eq!(result, myholdings);
        result.clear();

        let mut dump2: Vec<u8> = Vec::new();
        for value in ctx.iter() {
            dump2.push(value);
        }
        assert_eq!(dump, dump2);
        ctx.clear_all();
        let mut writer = ctx.create_writer();
        for data in dump.chunks(500) {
            writer.write_bulk(&data).unwrap();
        }

        let mut dump2: Vec<u8> = Vec::new();
        for value in ctx.iter() {
            dump2.push(value);
        }

        assert_eq!(dump, dump2);
    }

    fn gen_tcp_frame(data: &[u8]) -> ModbusFrame {
        let mut frame: ModbusFrame = [0; 256];
        frame[0] = 0x77;
        frame[1] = 0x55;
        frame[2] = 0;
        frame[3] = 0;
        let len = (data.len() as u16).to_be_bytes();
        frame[4] = len[0];
        frame[5] = len[1];
        for (i, v) in data.iter().enumerate() {
            frame[i + 6] = *v;
        }
        return frame;
    }

    // also automatically checks server::guest_rtu_frame_len
    fn gen_rtu_frame(data: &[u8]) -> ModbusFrame {
        let mut frame: ModbusFrame = [0; 256];
        for (i, v) in data.iter().enumerate() {
            frame[i] = *v;
        }
        let len = data.len();
        let crc16 = State::<MODBUS>::calculate(data);
        let c = crc16.to_le_bytes();
        frame[len] = c[0];
        frame[len + 1] = c[1];
        assert_eq!(
            guess_frame_len(&frame, ModbusProto::Rtu).unwrap(),
            (len + 2) as u8
        );
        return frame;
    }

    fn check_rtu_response(result: &Vec<u8>, response: &[u8]) {
        let mut resp = Vec::new();
        let mut r = Vec::new();
        for i in 6..response.len() {
            resp.push(response[i]);
        }
        for i in 0..result.len() - 2 {
            r.push(result[i]);
        }
        assert_eq!(resp, r);
        resp.insert(0, 1);
        let result_crc = u16::from_le_bytes([result[result.len() - 2], result[result.len() - 1]]);
        assert_eq!(result_crc, State::<MODBUS>::calculate(r.as_slice()));
    }

    /*
    #[test]
    fn test_std_frame_fc01_fc02_fc03_fc04_unknown_function() {
        clear_all();
        let mut result = Vec::new();
        // read coils
        coil_set(5, true).unwrap();
        coil_set(7, true).unwrap();
        coil_set(9, true).unwrap();
        let request = [1, 1, 0, 5, 0, 5];
        let response = [0x77, 0x55, 0, 0, 0, 4, 1, 1, 1, 0x15];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        let mut frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // check rtu crc error
        frame[request.len() + 1] = ((frame[request.len() + 1] as u16) + 1) as u8;
        match process_frame(1, &frame, ModbusProto::Rtu, &mut result) {
            Ok(_) => panic!(),
            Err(e) => match e {
                ErrorKind::FrameCRCError => {}
                _ => panic!(),
            },
        }
        // check illegal_function
        let request = [1, 7, 0x27, 0xe, 0, 0xf];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x87, 1];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // check context oob
        let request = [1, 1, 0x27, 0xe, 0, 0xf];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x81, 2];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // check invalid length
        let request = [1, 1, 0, 5, 0, 5];
        let mut frame = gen_tcp_frame(&request);
        frame[5] = 2;
        match process_frame(1, &frame, ModbusProto::TcpUdp, &mut result) {
            Ok(_) => panic!(),
            Err(e) => match e {
                ErrorKind::FrameBroken => {}
                _ => panic!("{:?}", e),
            },
        }
        let mut frame = gen_tcp_frame(&request);
        frame[5] = 251;
        match process_frame(1, &frame, ModbusProto::TcpUdp, &mut result) {
            Ok(_) => panic!(),
            Err(e) => match e {
                ErrorKind::FrameBroken => {}
                _ => panic!("{:?}", e),
            },
        }
        let mut frame = gen_tcp_frame(&request);
        frame[3] = 22;
        match process_frame(1, &frame, ModbusProto::TcpUdp, &mut result) {
            Ok(_) => panic!(),
            Err(e) => match e {
                ErrorKind::FrameBroken => {}
                _ => panic!("{:?}", e),
            },
        }
        // read discretes
        discrete_set(10, true).unwrap();
        discrete_set(12, true).unwrap();
        discrete_set(16, true).unwrap();
        let frame = gen_tcp_frame(&[1, 2, 0, 5, 0, 0x10]);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(
            result.as_slice(),
            [0x77, 0x55, 0, 0, 0, 5, 1, 2, 2, 0xa0, 8]
        );
        // read holdings
        holding_set(2, 9977).unwrap();
        holding_set(4, 9543).unwrap();
        holding_set(7, 9522).unwrap();
        let request = [1, 3, 0, 0, 0, 0xb];
        let frame = gen_tcp_frame(&request);
        let response = [
            0x77, 0x55, 0, 0, 0, 0x19, 1, 3, 0x16, 0, 0, 0, 0, 0x26, 0xf9, 0, 0, 0x25, 0x47, 0, 0,
            0, 0, 0x25, 0x32, 0, 0, 0, 0, 0, 0,
        ];
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // read inputs
        input_set(280, 99).unwrap();
        input_set(281, 15923).unwrap();
        input_set(284, 54321).unwrap();
        let frame = gen_tcp_frame(&[1, 4, 1, 0x18, 0, 6]);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(
            result.as_slice(),
            [
                0x77, 0x55, 0, 0, 0, 0xf, 1, 4, 0xc, 0, 0x63, 0x3e, 0x33, 0, 0, 0, 0, 0xd4, 0x31,
                0, 0
            ]
        );
    }

    #[test]
    fn test_std_frame_fc05_fc06() {
        clear_all();
        let mut result = Vec::new();
        // write coil
        let request = [1, 5, 0, 0xb, 0xff, 0];
        let response = [0x77, 0x55, 0, 0, 0, 6, 1, 5, 0, 0xb, 0xff, 0];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        assert_eq!(coil_get(11).unwrap(), true);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // write coil broadcast tcp
        let request = [0, 5, 0, 0x5, 0xff, 0];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.len(), 0);
        assert_eq!(coil_get(5).unwrap(), true);
        let request = [0, 5, 0, 0x7, 0xff, 0];
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        assert_eq!(result.len(), 0);
        assert_eq!(coil_get(7).unwrap(), true);
        // write coil invalid data
        let request = [1, 5, 0, 0xb, 0xff, 1];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x85, 3];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        // write coil context oob
        let request = [1, 5, 0x99, 0x99, 0xff, 0];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x85, 2];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // write holding
        let request = [1, 6, 0, 0xc, 0x33, 0x55];
        let response = [0x77, 0x55, 0, 0, 0, 6, 1, 6, 0, 0xc, 0x33, 0x55];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        assert_eq!(holding_get(12).unwrap(), 0x3355);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // write holding context oob
        let request = [1, 6, 0xff, 0xc, 0x33, 0x55];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x86, 2];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
    }

    #[test]
    fn test_std_frame_fc15() {
        clear_all();
        let mut result = Vec::new();
        // write multiple coils
        let request = [1, 0xf, 1, 0x31, 0, 5, 1, 0x25]; // 6 bits in data but 5 coils
        let response = [0x77, 0x55, 0, 0, 0, 6, 1, 0xf, 01, 0x31, 0, 5];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        assert_eq!(coil_get(305).unwrap(), true);
        assert_eq!(coil_get(306).unwrap(), false);
        assert_eq!(coil_get(307).unwrap(), true);
        assert_eq!(coil_get(308).unwrap(), false);
        assert_eq!(coil_get(309).unwrap(), false);
        assert_eq!(coil_get(310).unwrap(), false);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // write coils context oob
        let request = [1, 0xf, 0x99, 0xe8, 0, 5, 1, 0x25]; // 6 bits in data but 5 coils
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x8f, 2];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
    }

    #[test]
    fn test_std_frame_fc16() {
        clear_all();
        let mut result = Vec::new();
        // write multiple holdings
        let request = [
            1, 0x10, 1, 0x2c, 0, 4, 8, 0x11, 0x22, 0x11, 0x33, 0x11, 0x55, 0x11, 0x99,
        ];
        let response = [0x77, 0x55, 0, 0, 0, 6, 1, 0x10, 1, 0x2c, 0, 4];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        assert_eq!(holding_get(300).unwrap(), 0x1122);
        assert_eq!(holding_get(301).unwrap(), 0x1133);
        assert_eq!(holding_get(302).unwrap(), 0x1155);
        assert_eq!(holding_get(303).unwrap(), 0x1199);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // write holdings context oob
        let request = [
            1, 0x10, 0x99, 0xe8, 0, 4, 8, 0x11, 0x22, 0x11, 0x33, 0x11, 0x55, 0x11, 0x99,
        ];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x90, 2];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
    }*/
}

#[cfg(test)]
#[cfg(feature = "nostd")]
mod tests {
    use super::server::context::*;
    use super::server::*;
    use super::ErrorKind;

    use fixedvec::FixedVec;
    use rand::Rng;
    use spin::RwLock;

    lazy_static! {
        pub static ref CTX: RwLock<ModbusContext> = RwLock::new(ModbusContext::new());
    }

    #[test]
    fn test_nostd_coil_get_set_bulk() {
        let mut ctx = CTX.write();
        let mut data_mem = alloc_stack!([bool; CONTEXT_SIZE]);
        let mut data = FixedVec::new(&mut data_mem);
        let mut result_mem = alloc_stack!([bool; CONTEXT_SIZE]);
        let mut result = FixedVec::new(&mut result_mem);
        data.push_all(&[true; 2]).unwrap();
        ctx.set_coils_bulk(5, &data.as_slice()).unwrap();
        ctx.get_coils_bulk(5, 2, &mut result).unwrap();
        assert_eq!(result, data);

        data.clear();
        result.clear();

        data.push_all(&[true; 18]).unwrap();
        ctx.set_coils_bulk(25, &data.as_slice()).unwrap();
        ctx.get_coils_bulk(25, 18, &mut result).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_nostd_get_holding_set_bulk() {
        let mut ctx = CTX.write();
        let mut data_mem = alloc_stack!([u16; CONTEXT_SIZE]);
        let mut data = FixedVec::new(&mut data_mem);
        let mut result_mem = alloc_stack!([u16; CONTEXT_SIZE]);
        let mut result = FixedVec::new(&mut result_mem);

        ctx.clear_holdings();

        data.push_all(&[0x77; 2]).unwrap();
        ctx.set_holdings_bulk(5, &data.as_slice()).unwrap();
        ctx.get_holdings_bulk(5, 2, &mut result).unwrap();
        assert_eq!(result, data);

        data.clear();
        result.clear();

        data.push_all(&[0x33; 18]).unwrap();
        ctx.set_holdings_bulk(25, &data.as_slice()).unwrap();
        ctx.get_holdings_bulk(25, 18, &mut result).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_nostd_get_bools_as_u8() {
        let mut ctx = CTX.write();
        let mut data_mem = alloc_stack!([bool; CONTEXT_SIZE]);
        let mut data = FixedVec::new(&mut data_mem);
        ctx.clear_coils();
        data.push_all(&[true, true, true, true, true, true, false, false])
            .unwrap();
        ctx.set_coils_bulk(0, &data.as_slice()).unwrap();
        let mut result_mem = alloc_stack!([u8; CONTEXT_SIZE / 8]);
        let mut result = FixedVec::new(&mut result_mem);
        ctx.get_coils_as_u8(0, 6, &mut result).unwrap();
        assert_eq!(*result.get(0).unwrap(), 0b00111111);
        result.clear();
        ctx.get_coils_as_u8(0, 5, &mut result).unwrap();
        assert_eq!(*result.get(0).unwrap(), 0b00011111);
        result.clear();
        data.clear();
        data.push_all(&[true, true, false, true, true, true, true, true])
            .unwrap();
        ctx.set_coils_bulk(0, &data.as_slice()).unwrap();
        let mut result_mem = alloc_stack!([u8; CONTEXT_SIZE / 8]);
        let mut result = FixedVec::new(&mut result_mem);
        ctx.get_coils_as_u8(0, 6, &mut result).unwrap();
        assert_eq!(*result.get(0).unwrap(), 0b00111011);
        result.clear();
        ctx.get_coils_as_u8(0, 5, &mut result).unwrap();
        assert_eq!(*result.get(0).unwrap(), 0b00011011);
        result.clear();
        data.clear();
        data.push_all(&[
            true, true, false, true, true, true, true, true, // byte 1
            true, true, true, true, false, false, true, false, // byte 2
            false, false, false, true, false, true, // byte 3
        ])
        .unwrap();
        ctx.set_coils_bulk(0, &data.as_slice()).unwrap();
        let mut result_mem = alloc_stack!([u8; CONTEXT_SIZE / 8]);
        let mut result = FixedVec::new(&mut result_mem);
        ctx.get_coils_as_u8(0, 22, &mut result).unwrap();
        assert_eq!(*result.get(0).unwrap(), 0b11111011);
        assert_eq!(*result.get(1).unwrap(), 0b01001111);
        assert_eq!(*result.get(2).unwrap(), 0b101000);
    }

    #[test]
    fn test_nostd_get_set_regs_as_u8() {
        let mut ctx = CTX.write();
        ctx.clear_holdings();
        let mut data_mem = alloc_stack!([u16; CONTEXT_SIZE]);
        let mut data = FixedVec::new(&mut data_mem);
        data.push_all(&[2, 45, 4559, 31, 394, 1, 9, 7, 0, 1, 9])
            .unwrap();
        ctx.set_holdings_bulk(0, &data.as_slice()).unwrap();
        let mut result_mem = alloc_stack!([u8; CONTEXT_SIZE]);
        let mut result = FixedVec::new(&mut result_mem);
        ctx.get_holdings_as_u8(0, data.len() as u16, &mut result)
            .unwrap();
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 2);
        for i in 0..10 {
            ctx.set_holding(i, 0).unwrap();
        }
        ctx.set_holdings_from_u8(0, &result.as_slice()).unwrap();
        let mut result_mem = alloc_stack!([u16; CONTEXT_SIZE]);
        let mut result = FixedVec::new(&mut result_mem);
        ctx.get_holdings_bulk(0, data.len() as u16, &mut result)
            .unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_nostd_get_set_bools_as_u8() {
        let mut ctx = CTX.write();
        ctx.clear_coils();
        let mut data_mem = alloc_stack!([bool; CONTEXT_SIZE]);
        let mut data = FixedVec::new(&mut data_mem);
        data.push_all(&[
            true, true, true, false, true, true, true, true, true, false, false, false, false,
            false,
        ])
        .unwrap();
        ctx.set_coils_bulk(0, &data.as_slice()).unwrap();
        ctx.set_coil(data.len() as u16, true).unwrap();
        ctx.set_coil(data.len() as u16 + 1, false).unwrap();
        ctx.set_coil(data.len() as u16 + 2, true).unwrap();
        let mut result_mem = alloc_stack!([u8; CONTEXT_SIZE]);
        let mut result = FixedVec::new(&mut result_mem);
        ctx.get_coils_as_u8(0, data.len() as u16, &mut result)
            .unwrap();
        ctx.set_coils_from_u8(0, data.len() as u16, &result.as_slice())
            .unwrap();
        let mut result_mem = alloc_stack!([bool; CONTEXT_SIZE]);
        let mut result = FixedVec::new(&mut result_mem);
        ctx.get_coils_bulk(0, data.len() as u16, &mut result)
            .unwrap();
        assert_eq!(result, data);
        result.clear();
        data.push(true).unwrap();
        data.push(false).unwrap();
        data.push(true).unwrap();
        ctx.get_coils_bulk(0, data.len() as u16, &mut result)
            .unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_nostd_dump_restore() {
        let mut ctx = CTX.write();
        let mut rng = rand::thread_rng();
        let mut coils_mem = alloc_stack!([bool; CONTEXT_SIZE]);
        let mut discretes_mem = alloc_stack!([bool; CONTEXT_SIZE]);
        let mut inputs_mem = alloc_stack!([u16; CONTEXT_SIZE]);
        let mut holdings_mem = alloc_stack!([u16; CONTEXT_SIZE]);
        let mut mycoils = FixedVec::new(&mut coils_mem);
        let mut mydiscretes = FixedVec::new(&mut discretes_mem);
        let mut myinputs = FixedVec::new(&mut inputs_mem);
        let mut myholdings = FixedVec::new(&mut holdings_mem);
        for _ in 0..CONTEXT_SIZE {
            mycoils.push(rng.gen()).unwrap();
            mydiscretes.push(rng.gen()).unwrap();
            myholdings.push(rng.gen()).unwrap();
            myinputs.push(rng.gen()).unwrap();
        }
        ctx.clear_all();
        ctx.set_coils_bulk(0, &mycoils.as_slice()).unwrap();
        ctx.set_discretes_bulk(0, &mydiscretes.as_slice()).unwrap();
        ctx.set_holdings_bulk(0, &myholdings.as_slice()).unwrap();
        ctx.set_inputs_bulk(0, &myinputs.as_slice()).unwrap();
        let mut dump_mem = alloc_stack!([u8; CONTEXT_SIZE * 17 / 4]);
        let mut dump = FixedVec::new(&mut dump_mem);
        for i in 0..CONTEXT_SIZE * 17 / 4 {
            dump.push(ctx.get_cell(i as u16).unwrap()).unwrap();
        }
        ctx.clear_all();
        let mut offset = 0;
        for value in &dump {
            ctx.set_cell(offset, *value).unwrap();
            offset = offset + 1;
        }
        let mut result_mem = alloc_stack!([bool; CONTEXT_SIZE]);
        let mut result = FixedVec::new(&mut result_mem);
        ctx.get_coils_bulk(0, CONTEXT_SIZE as u16, &mut result)
            .unwrap();
        assert_eq!(result, mycoils);
        result.clear();
        ctx.get_discretes_bulk(0, CONTEXT_SIZE as u16, &mut result)
            .unwrap();
        assert_eq!(result, mydiscretes);
        result.clear();

        let mut result_mem = alloc_stack!([u16; CONTEXT_SIZE]);
        let mut result = FixedVec::new(&mut result_mem);
        ctx.get_inputs_bulk(0, CONTEXT_SIZE as u16, &mut result)
            .unwrap();
        assert_eq!(result, myinputs);
        result.clear();
        ctx.get_holdings_bulk(0, CONTEXT_SIZE as u16, &mut result)
            .unwrap();
        assert_eq!(result, myholdings);
        result.clear();

        let mut dump2_mem = alloc_stack!([u8; CONTEXT_SIZE * 17 / 4]);
        let mut dump2 = FixedVec::new(&mut dump2_mem);
        for value in ctx.iter() {
            dump2.push(value).unwrap();
        }
        assert_eq!(dump, dump2);
    }

    fn gen_tcp_frame(data: &[u8]) -> ModbusFrame {
        let mut frame: ModbusFrame = [0; 256];
        frame[0] = 0x77;
        frame[1] = 0x55;
        frame[2] = 0;
        frame[3] = 0;
        let len = (data.len() as u16).to_be_bytes();
        frame[4] = len[0];
        frame[5] = len[1];
        for (i, v) in data.iter().enumerate() {
            frame[i + 6] = *v;
        }
        return frame;
    }

    //#[test]
    //fn test_nostd_frame() {
    //clear_all();
    //let mut result_mem = alloc_stack!([u8; 256]);
    //let mut result = FixedVec::new(&mut result_mem);
    //coil_set(5, true).unwrap();
    //coil_set(7, true).unwrap();
    //coil_set(9, true).unwrap();
    //let request = [1, 1, 0, 5, 0, 5];
    //let response = [0x77, 0x55, 0, 0, 0, 4, 1, 1, 1, 0x15];
    //let frame = gen_tcp_frame(&request);
    //process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
    //assert_eq!(result.as_slice(), response);
    //check result OOB
    //let mut result_mem = alloc_stack!([u8; 10]);
    //for i in 0..10 {
    //let mut result = FixedVec::new(&mut result_mem[..i]);
    //match process_frame(1, &frame, ModbusProto::TcpUdp, &mut result) {
    //Ok(_) => panic!("{:x?}", result),
    //Err(e) => match e {
    //ErrorKind::OOB => {}
    //_ => panic!("{:?}", e),
    //},
    //}
    //}
    //}
}
