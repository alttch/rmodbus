//! # rmodbus - Modbus for Rust
//! 
//! A framework to build fast and reliable Modbus-powered applications.
//! 
//! Cargo crate: <https://crates.io/crates/rmodbus>
//! 
//! ## What is rmodbus
//! 
//! rmodbus is not a yet another Modbus client/server. rmodbus is a set of tools to
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
//! * supports client/server frame processing for Modbus TCP/UDP, RTU and ASCII.
//! 
//! * server context can be easily managed, imported and exported
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
//! use lazy_static::lazy_static;
//! 
//! use std::sync::RwLock;
//! 
//! use rmodbus::{
//!     server::{context::ModbusContext, ModbusFrame},
//!     ModbusFrameBuf, ModbusProto,
//! };
//! 
//! lazy_static! {
//!     pub static ref CONTEXT: RwLock<ModbusContext> = RwLock::new(ModbusContext::new());
//! }
//! 
//! pub fn tcpserver(unit: u8, listen: &str) {
//!     let listener = TcpListener::bind(listen).unwrap();
//!     println!("listening started, ready to accept");
//!     for stream in listener.incoming() {
//!         thread::spawn(move || {
//!             println!("client connected");
//!             let mut stream = stream.unwrap();
//!             loop {
//!                 let mut buf: ModbusFrameBuf = [0; 256];
//!                 let mut response = Vec::new(); // for nostd use FixedVec with alloc [u8;256]
//!                 if stream.read(&mut buf).unwrap_or(0) == 0 {
//!                     return;
//!                 }
//!                 let mut frame = ModbusFrame::new(unit, &buf, ModbusProto::TcpUdp, &mut response);
//!                 if frame.parse().is_err() {
//!                     println!("server error");
//!                     return;
//!                 }
//!                 if frame.processing_required {
//!                     let result = match frame.readonly {
//!                         true => frame.process_read(&CONTEXT.read().unwrap()),
//!                         false => frame.process_write(&mut CONTEXT.write().unwrap()),
//!                     };
//!                     if result.is_err() {
//!                         println!("frame processing error");
//!                         return;
//!                     }
//!                 }
//!                 if frame.response_required {
//!                     frame.finalize_response().unwrap();
//!                     println!("{:x?}", response.as_slice());
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
//! cargo run --example app --features std
//! cargo run --example tcpserver --features std
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
//! Every time Modbus context is accessed, a context mutex must be locked. This
//! slows down a performance, but guarantees that the context always has valid data
//! after bulk-sets or after 32-bit data types were written. So make sure your
//! application locks context only when required and only for a short period time.
//! 
//! Take a look at simple PLC example:
//! 
//! ```rust,ignore
//! use std::fs::File;
//! use std::io::{Write};
//! 
//! use rmodbus::server::context::ModbusContext;
//! 
//! fn looping() {
//!     println!("Loop started");
//!     loop {
//!         // READ WORK MODES ETC
//!         let ctx = srv::CONTEXT.read().unwrap();
//!         let _param1 = ctx.get_holding(1000).unwrap();
//!         let _param2 = ctx.get_holdings_as_f32(1100).unwrap(); // ieee754 f32
//!         let _param3 = ctx.get_holdings_as_u32(1200).unwrap(); // u32
//!         let cmd = ctx.get_holding(1500).unwrap();
//!         drop(ctx);
//!         if cmd != 0 {
//!             println!("got command code {}", cmd);
//!             let mut ctx = srv::CONTEXT.write().unwrap();
//!             ctx.set_holding(1500, 0).unwrap();
//!             match cmd {
//!                 1 => {
//!                     println!("saving memory context");
//!                     let _ = save("/tmp/plc1.dat", &ctx).map_err(|_| {
//!                         eprintln!("unable to save context!");
//!                     });
//!                 }
//!                 _ => println!("command not implemented"),
//!             }
//!         }
//!         // ==============================================
//!         // DO SOME JOB
//!         // ..........
//!         // WRITE RESULTS
//!         let mut ctx = srv::CONTEXT.write().unwrap();
//!         ctx.set_coil(0, true).unwrap();
//!         ctx.set_holdings_bulk(10, &(vec![10, 20])).unwrap();
//!         ctx.set_inputs_from_f32(20, 935.77).unwrap();
//!     }
//! }
//! 
//! fn save(fname: &str, ctx: &ModbusContext) -> Result<(), std::io::Error> {
//!     let mut file = match File::create(fname) {
//!         Ok(v) => v,
//!         Err(e) => return Err(e),
//!     };
//!     for i in ctx.iter() {
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
//! ## Vectors
//! 
//! Some of rmodbus functions use vectors to store result. In std mode, either
//! standard std::vec::Vec or [FixedVec](https://crates.io/crates/fixedvec) can be
//! used. In nostd mode, only FixedVec is supported.
//! 
//! ## Modbus client
//! 
//! Modbus client is designed with same principles as the server: the crate gives
//! frame generator / processor, while the frames can be read / written with any
//! source and with any required way.
//! 
//! TCP client Example:
//! 
//! ```rust,ignore
//! use std::io::{Read, Write};
//! use std::net::TcpStream;
//! use std::time::Duration;
//! 
//! use rmodbus::{client::ModbusRequest, guess_response_frame_len, ModbusProto};
//! 
//! fn main() {
//!     let timeout = Duration::from_secs(1);
//! 
//!     // open TCP connection
//!     let mut stream = TcpStream::connect("localhost:5502").unwrap();
//!     stream.set_read_timeout(Some(timeout)).unwrap();
//!     stream.set_write_timeout(Some(timeout)).unwrap();
//! 
//!     // create request object
//!     let mut mreq = ModbusRequest::new(1, ModbusProto::TcpUdp);
//!     mreq.tr_id = 2; // just for test, default tr_id is 1
//! 
//!     // set 2 coils
//!     let mut request = Vec::new();
//!     mreq.generate_set_coils_bulk(0, &[true, true], &mut request)
//!         .unwrap();
//! 
//!     // write request to stream
//!     stream.write(&request).unwrap();
//! 
//!     // read first 6 bytes of response frame
//!     let mut buf = [0u8; 6];
//!     stream.read_exact(&mut buf).unwrap();
//!     let mut response = Vec::new();
//!     response.extend_from_slice(&buf);
//!     let len = guess_response_frame_len(&buf, ModbusProto::TcpUdp).unwrap();
//!     // read rest of response frame
//!     if len > 6 {
//!         let mut rest = vec![0u8; (len - 6) as usize];
//!         stream.read_exact(&mut rest).unwrap();
//!         response.extend(rest);
//!     }
//!     // check if frame has no Modbus error inside
//!     mreq.parse_ok(&response).unwrap();
//! 
//!     // get coil values back
//!     mreq.generate_get_coils(0, 2, &mut request).unwrap();
//!     stream.write(&request).unwrap();
//!     let mut buf = [0u8; 6];
//!     stream.read_exact(&mut buf).unwrap();
//!     let mut response = Vec::new();
//!     response.extend_from_slice(&buf);
//!     let len = guess_response_frame_len(&buf, ModbusProto::TcpUdp).unwrap();
//!     if len > 6 {
//!         let mut rest = vec![0u8; (len - 6) as usize];
//!         stream.read_exact(&mut rest).unwrap();
//!         response.extend(rest);
//!     }
//!     let mut data = Vec::new();
//!     // check if frame has no Modbus error inside and parse response bools into data vec
//!     mreq.parse_bool(&response, &mut data).unwrap();
//!     for i in 0..data.len() {
//!         println!("{} {}", i, data[i]);
//!     }
//! }
//! ```
//! 
//! ## Changelog
//! 
//! ### v0.5
//! 
//! * Common functions and structures moved to main crate module
//! 
//! * Modbus client
//! 
//! ### v0.4
//! 
//! * Modbus context is no longer created automatically and no mutex guard is
//!   provided by default. Use ModbusContext::new() to create context object and
//!   then use it as you wish - protect with any kind of Mutex, with RwLock or just
//!   put into UnsafeCell.
//! 
//! * Context SDK changes: all functions moved inside context, removed unnecessary
//!   ones, function args optimized.
//! 
//! * FixedVec support included by default, both in std and nostd.
//! 
//! * Added support for 64-bit integers
//! 
#![cfg_attr(feature = "nostd", no_std)]

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
#[allow(unused_imports)]
#[macro_use]
extern crate fixedvec;

pub trait VectorTrait<T: Copy> {
    fn add(&mut self, value: T) -> Result<(), ErrorKind>;
    fn add_bulk(&mut self, other: &[T]) -> Result<(), ErrorKind>;
    fn get_len(&self) -> usize;
    fn clear_all(&mut self);
    fn cut_end(&mut self, len_to_cut: usize, value: T);
    fn get_slice(&self) -> &[T];
    fn replace(&mut self, index: usize, value: T);
}

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
    fn replace(&mut self, index: usize, value: T) {
        self[index] = value;
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
    fn replace(&mut self, index: usize, value: T) {
        self[index] = value;
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ErrorKind {
    OOB,
    OOBContext,
    FrameBroken,
    FrameCRCError,
    IllegalFunction,
    IllegalDataAddress,
    IllegalDataValue,
    SlaveDeviceFailure,
    Acknowledge,
    SlaveDeviceBusy,
    NegativeAcknowledge,
    MemoryParityError,
    GatewayPathUnavailable,
    GatewayTargetFailed,
    CommunicationError,
    UnknownError,
    Utf8Error,
}

impl ErrorKind {
    pub fn from_modbus_error(code: u8) -> Self {
        use ErrorKind::*;
        match code {
            0x01 => IllegalFunction,
            0x02 => IllegalDataAddress,
            0x03 => IllegalDataValue,
            0x04 => SlaveDeviceFailure,
            0x05 => Acknowledge,
            0x06 => SlaveDeviceBusy,
            0x07 => NegativeAcknowledge,
            0x08 => MemoryParityError,
            0x09 => GatewayPathUnavailable,
            0x10 => GatewayTargetFailed,
            _ => UnknownError,
        }
    }
}

#[cfg(not(feature = "nostd"))]
impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg: &str = match self {
            ErrorKind::OOB => "OOB",
            ErrorKind::OOBContext => "OOBContext",
            ErrorKind::FrameBroken => "FrameBroken",
            ErrorKind::FrameCRCError => "FrameCRCError",
            ErrorKind::IllegalFunction => "IllegalFunction",
            ErrorKind::IllegalDataAddress => "IllegalDataAddress",
            ErrorKind::IllegalDataValue => "IllegalDataValue",
            ErrorKind::SlaveDeviceFailure => "SlaveDeviceFailure",
            ErrorKind::Acknowledge => "Acknowledge",
            ErrorKind::SlaveDeviceBusy => "SlaveDeviceBusy",
            ErrorKind::NegativeAcknowledge => "NegativeAcknowledge",
            ErrorKind::MemoryParityError => "MemoryParityError",
            ErrorKind::GatewayPathUnavailable => "GatewayPathUnavailable",
            ErrorKind::GatewayTargetFailed => "GatewayTargetFailed",
            ErrorKind::CommunicationError => "CommunicationError",
            ErrorKind::UnknownError => "UnknownError",
            ErrorKind::Utf8Error => "Utf8Error",
        };
        write!(f, "{}", msg)
    } // fn fmt
} // impl std::fmt::Display

#[cfg(not(feature = "nostd"))]
impl std::error::Error for ErrorKind {
    fn description(&self) -> &str {
        match self {
            ErrorKind::OOB => "OUT OF BUFFER",
            ErrorKind::OOBContext => "OUT OF BUFFER IN CONTEXT",
            ErrorKind::FrameBroken => "FRAME BROKEN",
            ErrorKind::FrameCRCError => "FRAME CRC ERROR",
            ErrorKind::IllegalFunction => "MODBUS ERROR CODE 01 - ILLEGAL FUNCTION",
            ErrorKind::IllegalDataAddress => "MODBUS ERROR CODE 02 - ILLEGAL DATA ADDRESS",
            ErrorKind::IllegalDataValue => "MODBUS ERROR CODE 03 - ILLEGAL DATA VALUE",
            ErrorKind::SlaveDeviceFailure => "MODBUS ERROR CODE 04 - SLAVE DEVICE FAILURE",
            ErrorKind::Acknowledge => "MODBUS ERROR CODE 05 - ACKNOWLEDGE",
            ErrorKind::SlaveDeviceBusy => "MODBUS ERROR CODE 06 - SLAVE DEVICE BUSY",
            ErrorKind::NegativeAcknowledge => "MODBUS ERROR CODE 07 - NEGATIVE ACKNOWLEDGE",
            ErrorKind::MemoryParityError => "MODBUS ERROR CODE 08 - MEMORY PARITY ERROR",
            ErrorKind::GatewayPathUnavailable => "MODBUS ERROR CODE 10 - GATEWAY PATH UNAVAILABLE",
            ErrorKind::GatewayTargetFailed => {
                "MODBUS ERROR CODE 11 - GATEWAY TARGET DEVICE FAILED TO RESPOND"
            }
            ErrorKind::CommunicationError => {
                "MODBUS ERROR CODE 21 - Response CRC did not match calculated CRC"
            }
            ErrorKind::UnknownError => "UNKNOWN MODBUS ERROR",
            ErrorKind::Utf8Error => "UTF8 CONVERTION ERROR",
        }
    } // fn description
} // impl std::error::Error

pub const MODBUS_GET_COILS: u8 = 1;
pub const MODBUS_GET_DISCRETES: u8 = 2;
pub const MODBUS_GET_HOLDINGS: u8 = 3;
pub const MODBUS_GET_INPUTS: u8 = 4;
pub const MODBUS_SET_COIL: u8 = 5;
pub const MODBUS_SET_HOLDING: u8 = 6;
pub const MODBUS_SET_COILS_BULK: u8 = 15;
pub const MODBUS_SET_HOLDINGS_BULK: u8 = 16;

/// Modbus protocol selection for frame processing
///
/// * for **TcpUdp**, Modbus TCP headers are parsed / added to replies
/// * for **Rtu**, frame checksums are verified / added to replies
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum ModbusProto {
    Rtu,
    Ascii,
    TcpUdp,
}

/// Standard Modbus frame buffer
///
/// As max length of Modbus frame + headers is always 256 bytes or less, the frame buffer is a
/// fixed [u8; 256] array.
pub type ModbusFrameBuf = [u8; 256];

pub const MODBUS_ERROR_ILLEGAL_FUNCTION: u8 = 1;
pub const MODBUS_ERROR_ILLEGAL_DATA_ADDRESS: u8 = 2;
pub const MODBUS_ERROR_ILLEGAL_DATA_VALUE: u8 = 3;

/// Parse ASCII Modbus frame
///
/// data - input buffer
/// data_len - how many bytes to parse in buffer
/// frame_buf - frame buffer to write output
/// frame_pos - position in frame buffer to write
///
/// The frame can be parsed fully or partially (use frame_pos)
///
/// Returns number of bytes parsed
///
/// Errors:
///
/// * **OOB** input is larger than frame buffer (starting from frame_pos)
/// * **FrameBroken** unable to decode input hex string
pub fn parse_ascii_frame(
    data: &[u8],
    data_len: usize,
    frame_buf: &mut ModbusFrameBuf,
    frame_pos: u8,
) -> Result<u8, ErrorKind> {
    let mut dpos = match data[0] {
        58 => 1, // ':'
        _ => 0,
    };
    let mut cpos = frame_pos;
    while dpos < data_len {
        if cpos == 255 {
            return Err(ErrorKind::OOB);
        }
        let ch = data[dpos];
        if ch == 10 || ch == 13 || ch == 0 {
            break;
        }
        let c = match chr_to_hex(data[dpos]) {
            Ok(v) => v,
            Err(_) => return Err(ErrorKind::FrameBroken),
        };
        dpos = dpos + 1;
        if dpos >= data_len {
            return Err(ErrorKind::OOB);
        }
        let c2 = match chr_to_hex(data[dpos]) {
            Ok(v) => v,
            Err(_) => return Err(ErrorKind::FrameBroken),
        };
        frame_buf[cpos as usize] = c * 0x10 + c2;
        dpos = dpos + 1;
        cpos = cpos + 1;
    }
    return Ok(cpos - frame_pos);
}

/// Generate ASCII frame
///
/// Generates ASCII frame from binary response, made by "process_frame" function (response must be
/// supplited as slice)
pub fn generate_ascii_frame<V: VectorTrait<u8>>(
    data: &[u8],
    result: &mut V,
) -> Result<(), ErrorKind> {
    result.clear_all();
    if result.add(58).is_err() {
        return Err(ErrorKind::OOB);
    }
    for d in data {
        if result.add(hex_to_chr(d >> 4)).is_err() {
            return Err(ErrorKind::OOB);
        }
        if result.add(hex_to_chr(*d & 0xf)).is_err() {
            return Err(ErrorKind::OOB);
        }
    }
    if result.add(0x0D).is_err() {
        return Err(ErrorKind::OOB);
    }
    if result.add(0x0A).is_err() {
        return Err(ErrorKind::OOB);
    }
    return Ok(());
}

fn calc_crc16(frame: &[u8], data_length: u8) -> u16 {
    let mut crc: u16 = 0xffff;
    for pos in 0..data_length as usize {
        crc = crc ^ frame[pos] as u16;
        for _ in (0..8).rev() {
            if (crc & 0x0001) != 0 {
                crc = crc >> 1;
                crc = crc ^ 0xA001;
            } else {
                crc = crc >> 1;
            }
        }
    }
    return crc;
}

fn calc_lrc(frame: &[u8], data_length: u8) -> u8 {
    let mut lrc: i32 = 0;
    for i in 0..data_length {
        lrc = lrc - frame[i as usize] as i32;
    }
    return lrc as u8;
}

fn chr_to_hex(c: u8) -> Result<u8, ErrorKind> {
    if c >= 48 && c <= 57 {
        return Ok(c - 48);
    } else if c >= 65 && c <= 70 {
        return Ok(c - 55);
    } else {
        return Err(ErrorKind::FrameBroken);
    }
}

fn hex_to_chr(h: u8) -> u8 {
    if h < 10 {
        return h + 48;
    } else {
        return h + 55;
    }
}

/// Guess response frame length
///
/// Frames are often read byte-by-byte. The function allows to guess total frame length, having
/// first 7 (or more) bytes read.
///
/// How to use: read at least first 7 bytes (16 for ASCII) into buffer and call the function to
/// guess the total frame length. The remaining amount of bytes to read will be function result -
/// 7. 8 bytes is also fine, as that's the minimal correct frame length.
///
/// * the function will panic if the buffer length is less than 6 (3 for RTU, 7 for ASCII)
///
/// * the function may return wrong result for broken frames
///
/// * the function may return ErrorKind::FrameBroken for broken ASCII frames
pub fn guess_response_frame_len(buf: &[u8], proto: ModbusProto) -> Result<u8, ErrorKind> {
    let mut b: ModbusFrameBuf = [0; 256];
    let (f, multiplier, extra) = match proto {
        ModbusProto::Ascii => {
            if parse_ascii_frame(buf, buf.len(), &mut b, 0).is_err() {
                return Err(ErrorKind::FrameBroken);
            }
            (&b[0..256], 2, 5) // : + two chars LRC + \r\n
        }
        ModbusProto::TcpUdp => {
            let proto = u16::from_be_bytes([buf[2], buf[3]]);
            return match proto {
                0 => Ok((u16::from_be_bytes([buf[4], buf[5]]) + 6) as u8),
                _ => Err(ErrorKind::FrameBroken),
            };
        }
        ModbusProto::Rtu => (buf, 1, 2), // two bytes CRC16
    };
    let func = f[1];
    return match func < 0x80 {
        true => match func {
            1 | 2 | 3 | 4 => Ok((f[2] + 3) * multiplier + extra),
            5 | 6 | 15 | 16 => Ok(6 * multiplier + extra),
            _ => Err(ErrorKind::FrameBroken),
        },
        false => Ok(3 * multiplier + extra),
    };
}

/// Guess request frame length
///
/// Frames are often read byte-by-byte. The function allows to guess total frame length, having
/// first 7 (or more) bytes read.
///
/// How to use: read at least first 7 bytes (16 for ASCII) into buffer and call the function to
/// guess the total frame length. The remaining amount of bytes to read will be function result -
/// 7. 8 bytes is also fine, as that's the minimal correct frame length.
///
/// * the function will panic if the buffer length is less than 7 (for ASCII - 16)
///
/// * the function may return wrong result for broken frames
///
/// * the function may return ErrorKind::FrameBroken for broken ASCII frames
pub fn guess_request_frame_len(frame: &[u8], proto: ModbusProto) -> Result<u8, ErrorKind> {
    let mut buf: ModbusFrameBuf = [0; 256];
    let f;
    let extra;
    let multiplier;
    match proto {
        ModbusProto::Rtu => {
            f = frame;
            extra = 2;
            multiplier = 1;
        }
        ModbusProto::Ascii => match parse_ascii_frame(&frame, frame.len(), &mut buf, 0) {
            Ok(_) => {
                f = &buf;
                extra = 5;
                multiplier = 2;
            }
            Err(e) => return Err(e),
        },
        ModbusProto::TcpUdp => unimplemented!("unable to guess frame length for TCP/UDP"),
    };
    return match f[1] {
        15 | 16 => Ok((f[6] + 7) * multiplier + extra),
        _ => Ok(6 * multiplier + extra),
    };
}

#[path = "server.rs"]
pub mod server;

#[path = "client.rs"]
pub mod client;

#[cfg(test)]
#[cfg(not(feature = "nostd"))]
mod tests {
    use super::client::*;
    use super::server::context::*;
    use super::server::*;
    use super::*;

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

    fn gen_tcp_frame(data: &[u8]) -> ModbusFrameBuf {
        let mut frame: ModbusFrameBuf = [0; 256];
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
    fn gen_rtu_frame(data: &[u8]) -> ModbusFrameBuf {
        let mut frame: ModbusFrameBuf = [0; 256];
        for (i, v) in data.iter().enumerate() {
            frame[i] = *v;
        }
        let len = data.len();
        let crc16 = State::<MODBUS>::calculate(data);
        let c = crc16.to_le_bytes();
        frame[len] = c[0];
        frame[len + 1] = c[1];
        assert_eq!(
            guess_request_frame_len(&frame, ModbusProto::Rtu).unwrap(),
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

    #[test]
    fn test_std_frame_fc01_fc02_fc03_fc04_unknown_function() {
        let mut ctx = CTX.write().unwrap();
        ctx.clear_all();
        let mut result = Vec::new();

        // read coils
        ctx.set_coil(5, true).unwrap();
        ctx.set_coil(7, true).unwrap();
        ctx.set_coil(9, true).unwrap();
        let request = [1, 1, 0, 5, 0, 5];
        let response = [0x77, 0x55, 0, 0, 0, 4, 1, 1, 1, 0x15];
        let framebuf = gen_tcp_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.readonly, true);
        assert_eq!(frame.count, 5);
        assert_eq!(frame.reg, 5);
        assert_eq!(frame.error, 0);
        frame.process_read(&ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        assert_eq!(result.as_slice(), response);

        let mut framebuf = gen_rtu_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.readonly, true);
        assert_eq!(frame.count, 5);
        assert_eq!(frame.reg, 5);
        assert_eq!(frame.error, 0);
        frame.process_read(&ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        check_rtu_response(&result, &response);
        // check rtu crc error
        framebuf[request.len() + 1] = ((framebuf[request.len() + 1] as u16) + 1) as u8;
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
        match frame.parse() {
            Ok(_) => panic!(),
            Err(e) => match e {
                ErrorKind::FrameCRCError => {}
                _ => panic!(),
            },
        }
        // check illegal_function
        let request = [1, 7, 0x27, 0xe, 0, 0xf];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x87, 1];
        let framebuf = gen_tcp_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, false);
        assert_eq!(frame.error, 1);
        frame.finalize_response().unwrap();
        assert_eq!(result.as_slice(), response);

        let framebuf = gen_rtu_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, false);
        assert_eq!(frame.error, 1);
        frame.finalize_response().unwrap();
        check_rtu_response(&result, &response);
        // check context oob
        let request = [1, 1, 0x27, 0xe, 0, 0xf];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x81, 2];
        let framebuf = gen_tcp_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        frame.process_read(&ctx).unwrap();
        assert_eq!(frame.error, 2);
        frame.finalize_response().unwrap();
        assert_eq!(result.as_slice(), response);
        let framebuf = gen_rtu_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        frame.process_read(&ctx).unwrap();
        assert_eq!(frame.error, 2);
        frame.finalize_response().unwrap();
        check_rtu_response(&result, &response);
        // check invalid length
        let request = [1, 1, 0, 5, 0, 5];
        let mut framebuf = gen_tcp_frame(&request);
        framebuf[5] = 2;
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        match frame.parse() {
            Ok(_) => panic!(),
            Err(e) => match e {
                ErrorKind::FrameBroken => {}
                _ => panic!("{:?}", e),
            },
        }
        let mut framebuf = gen_tcp_frame(&request);
        framebuf[5] = 251;
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        match frame.parse() {
            Ok(_) => panic!(),
            Err(e) => match e {
                ErrorKind::FrameBroken => {}
                _ => panic!("{:?}", e),
            },
        }
        let mut framebuf = gen_tcp_frame(&request);
        framebuf[3] = 22;
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        match frame.parse() {
            Ok(_) => panic!(),
            Err(e) => match e {
                ErrorKind::FrameBroken => {}
                _ => panic!("{:?}", e),
            },
        }
        // read discretes
        ctx.set_discrete(10, true).unwrap();
        ctx.set_discrete(12, true).unwrap();
        ctx.set_discrete(16, true).unwrap();
        let framebuf = gen_tcp_frame(&[1, 2, 0, 5, 0, 0x10]);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, true);
        frame.process_read(&ctx).unwrap();
        frame.finalize_response().unwrap();
        assert_eq!(
            result.as_slice(),
            [0x77, 0x55, 0, 0, 0, 5, 1, 2, 2, 0xa0, 8]
        );
        // read holdings
        ctx.set_holding(2, 9977).unwrap();
        ctx.set_holding(4, 9543).unwrap();
        ctx.set_holding(7, 9522).unwrap();
        let request = [1, 3, 0, 0, 0, 0xb];
        let framebuf = gen_tcp_frame(&request);
        let response = [
            0x77, 0x55, 0, 0, 0, 0x19, 1, 3, 0x16, 0, 0, 0, 0, 0x26, 0xf9, 0, 0, 0x25, 0x47, 0, 0,
            0, 0, 0x25, 0x32, 0, 0, 0, 0, 0, 0,
        ];
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, true);
        frame.process_read(&ctx).unwrap();
        frame.finalize_response().unwrap();
        assert_eq!(result.as_slice(), response);
        let framebuf = gen_rtu_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, true);
        frame.process_read(&ctx).unwrap();
        frame.finalize_response().unwrap();
        check_rtu_response(&result, &response);
        // read inputs
        ctx.set_input(280, 99).unwrap();
        ctx.set_input(281, 15923).unwrap();
        ctx.set_input(284, 54321).unwrap();
        let framebuf = gen_tcp_frame(&[1, 4, 1, 0x18, 0, 6]);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, true);
        frame.process_read(&ctx).unwrap();
        frame.finalize_response().unwrap();
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
        let mut ctx = CTX.write().unwrap();
        ctx.clear_all();
        let mut result = Vec::new();
        // write coil
        let request = [1, 5, 0, 0xb, 0xff, 0];
        let response = [0x77, 0x55, 0, 0, 0, 6, 1, 5, 0, 0xb, 0xff, 0];
        let framebuf = gen_tcp_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        assert_eq!(result.as_slice(), response);
        assert_eq!(ctx.get_coil(11).unwrap(), true);
        let framebuf = gen_rtu_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        check_rtu_response(&result, &response);
        // write coil broadcast tcp
        let request = [0, 5, 0, 0x5, 0xff, 0];
        let framebuf = gen_tcp_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, false);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 0);
        assert_eq!(ctx.get_coil(5).unwrap(), true);
        let request = [0, 5, 0, 0x7, 0xff, 0];
        let framebuf = gen_rtu_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, false);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 0);
        assert_eq!(ctx.get_coil(7).unwrap(), true);
        // write coil invalid data
        let request = [1, 5, 0, 0xb, 0xff, 1];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x85, 3];
        let framebuf = gen_tcp_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 3);
        frame.finalize_response().unwrap();
        assert_eq!(result.as_slice(), response);
        // write coil context oob
        let request = [1, 5, 0x99, 0x99, 0xff, 0];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x85, 2];
        let framebuf = gen_tcp_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 2);
        frame.finalize_response().unwrap();
        assert_eq!(result.as_slice(), response);
        let framebuf = gen_rtu_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        frame.finalize_response().unwrap();
        check_rtu_response(&result, &response);
        // write holding
        let request = [1, 6, 0, 0xc, 0x33, 0x55];
        let response = [0x77, 0x55, 0, 0, 0, 6, 1, 6, 0, 0xc, 0x33, 0x55];
        let framebuf = gen_tcp_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        assert_eq!(result.as_slice(), response);
        assert_eq!(ctx.get_holding(12).unwrap(), 0x3355);
        let framebuf = gen_rtu_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        check_rtu_response(&result, &response);
        // write holding context oob
        let request = [1, 6, 0xff, 0xc, 0x33, 0x55];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x86, 2];
        let framebuf = gen_tcp_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 2);
        frame.finalize_response().unwrap();
        assert_eq!(result.as_slice(), response);
        let framebuf = gen_rtu_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 2);
        frame.finalize_response().unwrap();
        check_rtu_response(&result, &response);
    }

    #[test]
    fn test_std_frame_fc15() {
        let mut ctx = CTX.write().unwrap();
        ctx.clear_all();
        let mut result = Vec::new();
        // write multiple coils
        let request = [1, 0xf, 1, 0x31, 0, 5, 1, 0x25]; // 6 bits in data but 5 coils
        let response = [0x77, 0x55, 0, 0, 0, 6, 1, 0xf, 01, 0x31, 0, 5];
        let framebuf = gen_tcp_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        assert_eq!(result.as_slice(), response);
        assert_eq!(ctx.get_coil(305).unwrap(), true);
        assert_eq!(ctx.get_coil(306).unwrap(), false);
        assert_eq!(ctx.get_coil(307).unwrap(), true);
        assert_eq!(ctx.get_coil(308).unwrap(), false);
        assert_eq!(ctx.get_coil(309).unwrap(), false);
        assert_eq!(ctx.get_coil(310).unwrap(), false);
        let framebuf = gen_rtu_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        check_rtu_response(&result, &response);
        // write coils context oob
        let request = [1, 0xf, 0x99, 0xe8, 0, 5, 1, 0x25]; // 6 bits in data but 5 coils
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x8f, 2];
        let framebuf = gen_tcp_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 2);
        frame.finalize_response().unwrap();
        assert_eq!(result.as_slice(), response);
        let framebuf = gen_rtu_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 2);
        frame.finalize_response().unwrap();
        check_rtu_response(&result, &response);
    }

    #[test]
    fn test_std_frame_fc16() {
        let mut ctx = CTX.write().unwrap();
        ctx.clear_all();
        let mut result = Vec::new();
        // write multiple holdings
        let request = [
            1, 0x10, 1, 0x2c, 0, 4, 8, 0x11, 0x22, 0x11, 0x33, 0x11, 0x55, 0x11, 0x99,
        ];
        let response = [0x77, 0x55, 0, 0, 0, 6, 1, 0x10, 1, 0x2c, 0, 4];
        let framebuf = gen_tcp_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.func, 0x10);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        assert_eq!(result.as_slice(), response);
        assert_eq!(ctx.get_holding(300).unwrap(), 0x1122);
        assert_eq!(ctx.get_holding(301).unwrap(), 0x1133);
        assert_eq!(ctx.get_holding(302).unwrap(), 0x1155);
        assert_eq!(ctx.get_holding(303).unwrap(), 0x1199);
        let framebuf = gen_rtu_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        check_rtu_response(&result, &response);
        // write holdings context oob
        let request = [
            1, 0x10, 0x99, 0xe8, 0, 4, 8, 0x11, 0x22, 0x11, 0x33, 0x11, 0x55, 0x11, 0x99,
        ];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x90, 2];
        let framebuf = gen_tcp_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 2);
        frame.finalize_response().unwrap();
        assert_eq!(result.as_slice(), response);
        let framebuf = gen_rtu_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 2);
        frame.finalize_response().unwrap();
        check_rtu_response(&result, &response);
    }

    #[test]
    fn test_modbus_ascii() {
        let ctx = CTX.read().unwrap();
        let mut result = Vec::new();
        let mut ascii_result = Vec::new();
        let request = [
            0x3a, 0x30, 0x31, 0x30, 0x33, 0x30, 0x30, 0x30, 0x32, 0x30, 0x30, 0x30, 0x31, 0x46,
            0x39, 0xd, 0xa,
        ];
        let response = [
            0x3a, 0x30, 0x31, 0x30, 0x33, 0x30, 0x32, 0x30, 0x30, 0x30, 0x30, 0x46, 0x41, 0xd, 0xa,
        ];
        let mut framebuf: ModbusFrameBuf = [0; 256];
        parse_ascii_frame(&request, request.len(), &mut framebuf, 0).unwrap();
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Ascii, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, true);
        frame.process_read(&ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        generate_ascii_frame(&result.as_slice(), &mut ascii_result).unwrap();
        assert_eq!(ascii_result.as_slice(), response);
    }

    #[test]
    fn test_std_client() {
        let mut ctx = CTX.write().unwrap();
        ctx.clear_discretes();
        let coils = [
            true, true, true, false, true, true, false, true, true, false, true,
        ];
        let holdings = [2345u16, 4723, 193, 3845, 8321, 1244, 8723, 2231, 48572];
        let holdstr = "The Quick Brown Fox Jumps Over The Lazy Dog";
        let protos = [ModbusProto::TcpUdp, ModbusProto::Rtu, ModbusProto::Ascii];

        for proto in &protos {
            // set coils bulk
            ctx.clear_coils();
            let mut mreq = ModbusRequest::new(2, *proto);
            let mut request = Vec::new();
            mreq.generate_set_coils_bulk(100, &coils, &mut request)
                .unwrap();
            let mut response = Vec::new();
            let mut framebuf: ModbusFrameBuf = [0; 256];
            if *proto == ModbusProto::Rtu {
                let mut ascii_frame = Vec::new();
                generate_ascii_frame(&request, &mut ascii_frame).unwrap();
                for i in 0..framebuf.len() {
                    framebuf[i] = 0
                }
                parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
            } else {
                for i in 0..request.len() {
                    framebuf[i] = request[i];
                }
            }
            let mut frame = ModbusFrame::new(2, &framebuf, *proto, &mut response);
            frame.parse().unwrap();
            assert_eq!(frame.response_required, true);
            assert_eq!(frame.processing_required, true);
            assert_eq!(frame.error, 0);
            assert_eq!(frame.readonly, false);
            frame.process_write(&mut ctx).unwrap();
            assert_eq!(frame.error, 0);
            frame.finalize_response().unwrap();
            mreq.parse_ok(&response).unwrap();
            for i in 100..100 + coils.len() {
                assert_eq!(ctx.get_coil(i as u16).unwrap(), coils[i - 100]);
            }

            // reading coils
            let mut mreq = ModbusRequest::new(3, *proto);
            let mut request = Vec::new();
            mreq.generate_get_coils(100, coils.len() as u16, &mut request)
                .unwrap();
            let mut response = Vec::new();
            let mut framebuf: ModbusFrameBuf = [0; 256];
            if *proto == ModbusProto::Rtu {
                let mut ascii_frame = Vec::new();
                generate_ascii_frame(&request, &mut ascii_frame).unwrap();
                for i in 0..framebuf.len() {
                    framebuf[i] = 0
                }
                parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
            } else {
                for i in 0..request.len() {
                    framebuf[i] = request[i];
                }
            }
            let mut frame = ModbusFrame::new(3, &framebuf, *proto, &mut response);
            frame.parse().unwrap();
            assert_eq!(frame.response_required, true);
            assert_eq!(frame.processing_required, true);
            assert_eq!(frame.error, 0);
            assert_eq!(frame.readonly, true);
            frame.process_read(&mut ctx).unwrap();
            assert_eq!(frame.error, 0);
            frame.finalize_response().unwrap();
            let mut result = Vec::new();
            mreq.parse_bool(&response, &mut result).unwrap();
            assert_eq!(result, coils);

            // reading discretes
            ctx.clear_coils();
            ctx.clear_discretes();
            for c in 200..200 + coils.len() {
                ctx.set_discrete(c as u16, coils[c - 200]).unwrap();
            }
            let mut mreq = ModbusRequest::new(3, *proto);
            let mut request = Vec::new();
            mreq.generate_get_discretes(200, coils.len() as u16, &mut request)
                .unwrap();
            let mut response = Vec::new();
            let mut framebuf: ModbusFrameBuf = [0; 256];
            if *proto == ModbusProto::Rtu {
                let mut ascii_frame = Vec::new();
                generate_ascii_frame(&request, &mut ascii_frame).unwrap();
                for i in 0..framebuf.len() {
                    framebuf[i] = 0
                }
                parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
            } else {
                for i in 0..request.len() {
                    framebuf[i] = request[i];
                }
            }
            let mut frame = ModbusFrame::new(3, &framebuf, *proto, &mut response);
            frame.parse().unwrap();
            assert_eq!(frame.response_required, true);
            assert_eq!(frame.processing_required, true);
            assert_eq!(frame.error, 0);
            assert_eq!(frame.readonly, true);
            frame.process_read(&mut ctx).unwrap();
            assert_eq!(frame.error, 0);
            frame.finalize_response().unwrap();
            let mut result = Vec::new();
            mreq.parse_bool(&response, &mut result).unwrap();
            assert_eq!(result, coils);

            // set single coil
            ctx.clear_coils();
            ctx.clear_discretes();
            let mut mreq = ModbusRequest::new(4, *proto);
            let mut request = Vec::new();
            mreq.generate_set_coil(100, true, &mut request).unwrap();
            let mut response = Vec::new();
            let mut framebuf: ModbusFrameBuf = [0; 256];
            if *proto == ModbusProto::Rtu {
                let mut ascii_frame = Vec::new();
                generate_ascii_frame(&request, &mut ascii_frame).unwrap();
                for i in 0..framebuf.len() {
                    framebuf[i] = 0
                }
                parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
            } else {
                for i in 0..request.len() {
                    framebuf[i] = request[i];
                }
            }
            let mut frame = ModbusFrame::new(4, &framebuf, *proto, &mut response);
            frame.parse().unwrap();
            assert_eq!(frame.response_required, true);
            assert_eq!(frame.processing_required, true);
            assert_eq!(frame.error, 0);
            assert_eq!(frame.readonly, false);
            frame.process_write(&mut ctx).unwrap();
            assert_eq!(frame.error, 0);
            frame.finalize_response().unwrap();
            mreq.parse_ok(&response).unwrap();
            assert_eq!(ctx.get_coil(100).unwrap(), true);

            // set coils oob
            let mut mreq = ModbusRequest::new(4, *proto);
            let mut request = Vec::new();
            mreq.generate_set_coil(10001, true, &mut request).unwrap();
            let mut response = Vec::new();
            let mut framebuf: ModbusFrameBuf = [0; 256];
            if *proto == ModbusProto::Rtu {
                let mut ascii_frame = Vec::new();
                generate_ascii_frame(&request, &mut ascii_frame).unwrap();
                for i in 0..framebuf.len() {
                    framebuf[i] = 0
                }
                parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
            } else {
                for i in 0..request.len() {
                    framebuf[i] = request[i];
                }
            }
            let mut frame = ModbusFrame::new(4, &framebuf, *proto, &mut response);
            frame.parse().unwrap();
            assert_eq!(frame.response_required, true);
            assert_eq!(frame.processing_required, true);
            assert_eq!(frame.error, 0);
            assert_eq!(frame.readonly, false);
            frame.process_write(&mut ctx).unwrap();
            assert_eq!(frame.error, 2);
            frame.finalize_response().unwrap();
            assert_eq!(
                mreq.parse_ok(&response).err().unwrap(),
                ErrorKind::IllegalDataAddress
            );

            // set holdings bulk
            ctx.clear_holdings();
            let mut mreq = ModbusRequest::new(2, *proto);
            let mut request = Vec::new();
            mreq.generate_set_holdings_bulk(100, &holdings, &mut request)
                .unwrap();
            let mut response = Vec::new();
            let mut framebuf: ModbusFrameBuf = [0; 256];
            if *proto == ModbusProto::Rtu {
                let mut ascii_frame = Vec::new();
                generate_ascii_frame(&request, &mut ascii_frame).unwrap();
                for i in 0..framebuf.len() {
                    framebuf[i] = 0
                }
                parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
            } else {
                for i in 0..request.len() {
                    framebuf[i] = request[i];
                }
            }
            let mut frame = ModbusFrame::new(2, &framebuf, *proto, &mut response);
            frame.parse().unwrap();
            assert_eq!(frame.response_required, true);
            assert_eq!(frame.processing_required, true);
            assert_eq!(frame.error, 0);
            assert_eq!(frame.readonly, false);
            frame.process_write(&mut ctx).unwrap();
            assert_eq!(frame.error, 0);
            frame.finalize_response().unwrap();
            mreq.parse_ok(&response).unwrap();
            for i in 100..100 + holdings.len() {
                assert_eq!(ctx.get_holding(i as u16).unwrap(), holdings[i - 100]);
            }

            // reading holdings
            let mut mreq = ModbusRequest::new(3, *proto);
            let mut request = Vec::new();
            mreq.generate_get_holdings(100, holdings.len() as u16, &mut request)
                .unwrap();
            let mut response = Vec::new();
            let mut framebuf: ModbusFrameBuf = [0; 256];
            if *proto == ModbusProto::Rtu {
                let mut ascii_frame = Vec::new();
                generate_ascii_frame(&request, &mut ascii_frame).unwrap();
                for i in 0..framebuf.len() {
                    framebuf[i] = 0
                }
                parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
            } else {
                for i in 0..request.len() {
                    framebuf[i] = request[i];
                }
            }
            let mut frame = ModbusFrame::new(3, &framebuf, *proto, &mut response);
            frame.parse().unwrap();
            assert_eq!(frame.response_required, true);
            assert_eq!(frame.processing_required, true);
            assert_eq!(frame.error, 0);
            assert_eq!(frame.readonly, true);
            frame.process_read(&mut ctx).unwrap();
            assert_eq!(frame.error, 0);
            frame.finalize_response().unwrap();
            let mut result = Vec::new();
            mreq.parse_u16(&response, &mut result).unwrap();
            assert_eq!(result, holdings);

            // set holdings string
            ctx.clear_holdings();
            let mut mreq = ModbusRequest::new(2, *proto);
            let mut request = Vec::new();
            mreq.generate_set_holdings_string(100, &holdstr, &mut request)
                .unwrap();
            let mut response = Vec::new();
            let mut framebuf: ModbusFrameBuf = [0; 256];
            if *proto == ModbusProto::Rtu {
                let mut ascii_frame = Vec::new();
                generate_ascii_frame(&request, &mut ascii_frame).unwrap();
                for i in 0..framebuf.len() {
                    framebuf[i] = 0
                }
                parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
            } else {
                for i in 0..request.len() {
                    framebuf[i] = request[i];
                }
            }
            let mut frame = ModbusFrame::new(2, &framebuf, *proto, &mut response);
            frame.parse().unwrap();
            assert_eq!(frame.response_required, true);
            assert_eq!(frame.processing_required, true);
            assert_eq!(frame.error, 0);
            assert_eq!(frame.readonly, false);
            frame.process_write(&mut ctx).unwrap();
            assert_eq!(frame.error, 0);
            frame.finalize_response().unwrap();
            mreq.parse_ok(&response).unwrap();

            // reading holdings string
            let mut mreq = ModbusRequest::new(3, *proto);
            let mut request = Vec::new();
            mreq.generate_get_holdings(100, holdstr.len() as u16, &mut request)
                .unwrap();
            let mut response = Vec::new();
            let mut framebuf: ModbusFrameBuf = [0; 256];
            if *proto == ModbusProto::Rtu {
                let mut ascii_frame = Vec::new();
                generate_ascii_frame(&request, &mut ascii_frame).unwrap();
                for i in 0..framebuf.len() {
                    framebuf[i] = 0
                }
                parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
            } else {
                for i in 0..request.len() {
                    framebuf[i] = request[i];
                }
            }
            let mut frame = ModbusFrame::new(3, &framebuf, *proto, &mut response);
            frame.parse().unwrap();
            assert_eq!(frame.response_required, true);
            assert_eq!(frame.processing_required, true);
            assert_eq!(frame.error, 0);
            assert_eq!(frame.readonly, true);
            frame.process_read(&mut ctx).unwrap();
            assert_eq!(frame.error, 0);
            frame.finalize_response().unwrap();
            let mut result = String::new();
            mreq.parse_string(&response, &mut result).unwrap();
            assert_eq!(result, holdstr);

            // reading inputs
            ctx.clear_holdings();
            ctx.clear_inputs();
            for c in 200..200 + holdings.len() {
                ctx.set_input(c as u16, holdings[c - 200]).unwrap();
            }
            let mut mreq = ModbusRequest::new(3, *proto);
            let mut request = Vec::new();
            mreq.generate_get_inputs(200, holdings.len() as u16, &mut request)
                .unwrap();
            let mut response = Vec::new();
            let mut framebuf: ModbusFrameBuf = [0; 256];
            if *proto == ModbusProto::Rtu {
                let mut ascii_frame = Vec::new();
                generate_ascii_frame(&request, &mut ascii_frame).unwrap();
                for i in 0..framebuf.len() {
                    framebuf[i] = 0
                }
                parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
            } else {
                for i in 0..request.len() {
                    framebuf[i] = request[i];
                }
            }
            let mut frame = ModbusFrame::new(3, &framebuf, *proto, &mut response);
            frame.parse().unwrap();
            assert_eq!(frame.response_required, true);
            assert_eq!(frame.processing_required, true);
            assert_eq!(frame.error, 0);
            assert_eq!(frame.readonly, true);
            frame.process_read(&mut ctx).unwrap();
            assert_eq!(frame.error, 0);
            frame.finalize_response().unwrap();
            let mut result = Vec::new();
            mreq.parse_u16(&response, &mut result).unwrap();
            assert_eq!(result, holdings);

            // set single holding
            ctx.clear_holdings();
            ctx.clear_inputs();
            let mut mreq = ModbusRequest::new(4, *proto);
            let mut request = Vec::new();
            mreq.generate_set_holding(100, 7777, &mut request).unwrap();
            let mut response = Vec::new();
            let mut framebuf: ModbusFrameBuf = [0; 256];
            if *proto == ModbusProto::Rtu {
                let mut ascii_frame = Vec::new();
                generate_ascii_frame(&request, &mut ascii_frame).unwrap();
                for i in 0..framebuf.len() {
                    framebuf[i] = 0
                }
                parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
            } else {
                for i in 0..request.len() {
                    framebuf[i] = request[i];
                }
            }
            let mut frame = ModbusFrame::new(4, &framebuf, *proto, &mut response);
            frame.parse().unwrap();
            assert_eq!(frame.response_required, true);
            assert_eq!(frame.processing_required, true);
            assert_eq!(frame.error, 0);
            assert_eq!(frame.readonly, false);
            frame.process_write(&mut ctx).unwrap();
            assert_eq!(frame.error, 0);
            frame.finalize_response().unwrap();
            mreq.parse_ok(&response).unwrap();
            assert_eq!(ctx.get_coil(100).unwrap(), true);

            // set holding oob
            let mut mreq = ModbusRequest::new(4, *proto);
            let mut request = Vec::new();
            mreq.generate_set_holding(10001, 0x7777, &mut request)
                .unwrap();
            let mut response = Vec::new();
            let mut framebuf: ModbusFrameBuf = [0; 256];
            if *proto == ModbusProto::Rtu {
                let mut ascii_frame = Vec::new();
                generate_ascii_frame(&request, &mut ascii_frame).unwrap();
                for i in 0..framebuf.len() {
                    framebuf[i] = 0
                }
                parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
            } else {
                for i in 0..request.len() {
                    framebuf[i] = request[i];
                }
            }
            let mut frame = ModbusFrame::new(4, &framebuf, *proto, &mut response);
            frame.parse().unwrap();
            assert_eq!(frame.response_required, true);
            assert_eq!(frame.processing_required, true);
            assert_eq!(frame.error, 0);
            assert_eq!(frame.readonly, false);
            frame.process_write(&mut ctx).unwrap();
            assert_eq!(frame.error, 2);
            frame.finalize_response().unwrap();
            assert_eq!(
                mreq.parse_ok(&response).err().unwrap(),
                ErrorKind::IllegalDataAddress
            );
        }
    }
}

#[cfg(test)]
#[cfg(feature = "nostd")]
mod tests {
    use super::client::*;
    use super::server::context::*;
    use super::server::*;
    use super::*;

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

    fn gen_tcp_frame(data: &[u8]) -> ModbusFrameBuf {
        let mut frame: ModbusFrameBuf = [0; 256];
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

    #[test]
    fn test_nostd_frame() {
        let mut ctx = CTX.write();
        ctx.clear_coils();
        ctx.clear_all();
        let mut result_mem = alloc_stack!([u8; 256]);
        let mut result = FixedVec::new(&mut result_mem);
        ctx.set_coil(5, true).unwrap();
        ctx.set_coil(7, true).unwrap();
        ctx.set_coil(9, true).unwrap();
        let request = [1, 1, 0, 5, 0, 5];
        let response = [0x77, 0x55, 0, 0, 0, 4, 1, 1, 1, 0x15];
        let framebuf = gen_tcp_frame(&request);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, true);
        frame.process_read(&ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        assert_eq!(result.as_slice(), response);
        //check result OOB
        let mut result_mem = alloc_stack!([u8; 10]);
        for i in 0..10 {
            let mut result = FixedVec::new(&mut result_mem[..i]);
            let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
            match frame.parse() {
                Ok(_) => {
                    if i > 3 {
                        match frame.process_read(&ctx) {
                            Ok(_) => panic!("{:x?}", result),
                            Err(e) => match e {
                                ErrorKind::OOB => {}
                                _ => panic!("{:?}", e),
                            },
                        }
                    } else {
                        panic!("{:x?}", result)
                    }
                }
                Err(e) => match e {
                    ErrorKind::OOB => {}
                    _ => panic!("{:?}", e),
                },
            }
        }
    }

    #[test]
    fn test_nostd_client() {
        let mut ctx = CTX.write();
        ctx.clear_discretes();
        let coils = [
            true, true, true, false, true, true, false, true, true, false, true,
        ];
        let protos = [ModbusProto::TcpUdp, ModbusProto::Rtu, ModbusProto::Ascii];

        for proto in &protos {
            // set coils bulk
            ctx.clear_coils();
            let mut mreq = ModbusRequest::new(2, *proto);
            let mut request_mem = alloc_stack!([u8; 256]);
            let mut request = FixedVec::new(&mut request_mem);
            mreq.generate_set_coils_bulk(100, &coils, &mut request)
                .unwrap();
            let mut response_mem = alloc_stack!([u8; 256]);
            let mut response = FixedVec::new(&mut response_mem);
            let mut framebuf: ModbusFrameBuf = [0; 256];
            if *proto == ModbusProto::Rtu {
                let mut ascii_mem = alloc_stack!([u8; 1024]);
                let mut ascii_frame = FixedVec::new(&mut ascii_mem);
                generate_ascii_frame(&request.as_slice(), &mut ascii_frame).unwrap();
                for i in 0..framebuf.len() {
                    framebuf[i] = 0
                }
                parse_ascii_frame(&ascii_frame.as_slice(), ascii_frame.len(), &mut framebuf, 0)
                    .unwrap();
            } else {
                for i in 0..request.len() {
                    framebuf[i] = request[i];
                }
            }
            let mut frame = ModbusFrame::new(2, &framebuf, *proto, &mut response);
            frame.parse().unwrap();
            assert_eq!(frame.response_required, true);
            assert_eq!(frame.processing_required, true);
            assert_eq!(frame.error, 0);
            assert_eq!(frame.readonly, false);
            frame.process_write(&mut ctx).unwrap();
            assert_eq!(frame.error, 0);
            frame.finalize_response().unwrap();
            mreq.parse_ok(&response.as_slice()).unwrap();
            for i in 100..100 + coils.len() {
                assert_eq!(ctx.get_coil(i as u16).unwrap(), coils[i - 100]);
            }

            // reading coils
            let mut mreq = ModbusRequest::new(3, *proto);
            let mut request_mem = alloc_stack!([u8; 256]);
            let mut request = FixedVec::new(&mut request_mem);
            mreq.generate_get_coils(100, coils.len() as u16, &mut request)
                .unwrap();
            let mut response_mem = alloc_stack!([u8; 256]);
            let mut response = FixedVec::new(&mut response_mem);
            let mut framebuf: ModbusFrameBuf = [0; 256];
            if *proto == ModbusProto::Rtu {
                let mut ascii_mem = alloc_stack!([u8; 1024]);
                let mut ascii_frame = FixedVec::new(&mut ascii_mem);
                generate_ascii_frame(&request.as_slice(), &mut ascii_frame).unwrap();
                for i in 0..framebuf.len() {
                    framebuf[i] = 0
                }
                parse_ascii_frame(&ascii_frame.as_slice(), ascii_frame.len(), &mut framebuf, 0)
                    .unwrap();
            } else {
                for i in 0..request.len() {
                    framebuf[i] = request[i];
                }
            }
            let mut frame = ModbusFrame::new(3, &framebuf, *proto, &mut response);
            frame.parse().unwrap();
            assert_eq!(frame.response_required, true);
            assert_eq!(frame.processing_required, true);
            assert_eq!(frame.error, 0);
            assert_eq!(frame.readonly, true);
            frame.process_read(&mut ctx).unwrap();
            assert_eq!(frame.error, 0);
            frame.finalize_response().unwrap();
            let mut result_mem = alloc_stack!([bool; 256]);
            let mut result = FixedVec::new(&mut result_mem);
            mreq.parse_bool(&response.as_slice(), &mut result).unwrap();
            assert_eq!(result.as_slice(), coils);
        }
    }
}
