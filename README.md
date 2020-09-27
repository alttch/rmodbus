# rmodbus - Modbus for Rust

A framework to build fast and reliable Modbus-powered applications.

Cargo crate: https://crates.io/crates/rmodbus

## What is rmodbus

rmodbus is not a yet another Modbus server. rmodbus is a set of tools to
quickly build Modbus-powered applications.

## Why yet another Modbus lib?

* rmodbus is transport and protocol independent

* rmodbus is platform independent (**no\_std is fully supported!**)

* can be easily used in blocking and async (non-blocking) applications

* tuned for speed and reliability

* provides a set of tools to easily work with Modbus context

* supports server frame processing for Modbus TCP/UDP, RTU and ASCII.

* server context can be easily, managed, imported and exported

## So the server isn't included?

Yes, there's no server included. You build the server by your own. You choose
protocol, technology and everything else. rmodbus just process frames and works
with Modbus context.

Here's an example of a simple TCP blocking server:

```rust,ignore
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use lazy_static::lazy_static;

use std::sync::RwLock;

use rmodbus::server::{context::ModbusContext, ModbusFrame, ModbusFrameBuf, ModbusProto};

lazy_static! {
    pub static ref CONTEXT: RwLock<ModbusContext> = RwLock::new(ModbusContext::new());
}

pub fn tcpserver(unit: u8, listen: &str) {
    let listener = TcpListener::bind(listen).unwrap();
    println!("listening started, ready to accept");
    for stream in listener.incoming() {
        thread::spawn(move || {
            println!("client connected");
            let mut stream = stream.unwrap();
            loop {
                let mut buf: ModbusFrameBuf = [0; 256];
                let mut response = Vec::new(); // for nostd use FixedVec with alloc [u8;256]
                if stream.read(&mut buf).unwrap_or(0) == 0 {
                    return;
                }
                let mut frame = ModbusFrame::new(unit, &buf, ModbusProto::TcpUdp, &mut response);
                if frame.parse().is_err() {
                    println!("server error");
                    return;
                }
                if frame.processing_required {
                    let result = match frame.readonly {
                        true => frame.process_read(&CONTEXT.read().unwrap()),
                        false => frame.process_write(&mut CONTEXT.write().unwrap()),
                    };
                    if result.is_err() {
                        println!("frame processing error");
                        return;
                    }
                }
                if frame.response_required {
                    frame.finalize_response().unwrap();
                    println!("{:x?}", response.as_slice());
                    if stream.write(response.as_slice()).is_err() {
                        return;
                    }
                }
            }
        });
    }
}
```

There are also examples for Serial-RTU, Serial-ASCII and UDP in *examples*
folder (if you're reading this text somewhere else, visit [rmodbus project
repository](https://github.com/alttch/rmodbus).

Running examples:

```shell
cargo run --example app --features std
cargo run --example tcpserver --features std
```

## Modbus context

The rule is simple: one standard Modbus context per application. 10k+10k 16-bit
registers and 10k+10k coils are usually more than enough. This takes about
43Kbytes of RAM, but if you need to reduce context size, download library
source and change *CONTEXT_SIZE* constant in "context.rs".

rmodbus server context is thread-safe, easy to use and has a lot of functions.

The context is created automatically, as soon as the library is imported. No
additional action is required.

Every time Modbus context is accessed, a context mutex must be locked. This
slows down a performance, but guarantees that the context always has valid data
after bulk-sets or after 32-bit data types were written. So make sure your
application locks context only when required and only for a short period time.

There are two groups of context functions:

* High-level API: simple functions like *coil_get* automatically lock the
  context but do this every time when called. Use this for testing or if the
  speed is not important.

* Advanced way is to use low-level API, lock the context manually and then call
  proper functions, like *set*, *set_f32* etc.

Take a look at simple PLC example:

```rust,ignore
use std::fs::File;
use std::io::Write;

use lazy_static::lazy_static;

use std::sync::RwLock;

use rmodbus::server::context::ModbusContext;

lazy_static! {
    pub static ref CONTEXT: RwLock<ModbusContext> = RwLock::new(ModbusContext::new());
}

fn looping() {
    println!("Loop started");
    loop {
        // READ WORK MODES ETC
        let ctx = CONTEXT.read().unwrap();
        let _param1 = ctx.get_holding(1000).unwrap();
        let _param2 = ctx.get_holdings_as_f32(1100).unwrap(); // ieee754 f32
        let _param3 = ctx.get_holdings_as_u32(1200).unwrap(); // u32
        let cmd = ctx.get_holding(1500).unwrap();
        drop(ctx);
        if cmd != 0 {
            println!("got command code {}", cmd);
            let mut ctx = CONTEXT.write().unwrap();
            ctx.set_holding(1500, 0).unwrap();
            match cmd {
                1 => {
                    println!("saving memory context");
                    let _ = save("/tmp/plc1.dat", &ctx).map_err(|_| {
                        eprintln!("unable to save context!");
                    });
                }
                _ => println!("command not implemented"),
            }
        }
        // ==============================================
        // DO SOME JOB
        // ..........
        // WRITE RESULTS
        let mut ctx = CONTEXT.write().unwrap();
        ctx.set_coil(0, true).unwrap();
        ctx.set_holdings_bulk(10, &(vec![10, 20])).unwrap();
        ctx.set_inputs_from_f32(20, 935.77).unwrap();
    }
}

fn save(fname: &str, ctx: &ModbusContext) -> Result<(), std::io::Error> {
    let mut file = match File::create(fname) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    for i in ctx.iter() {
        match file.write(&[i]) {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
    }
    match file.sync_all() {
        Ok(_) => {}
        Err(e) => return Err(e),
    }
    return Ok(());
}
```

To let the above program communicate with outer world, Modbus server must be up
and running in the separate thread, asynchronously or whatever is preferred.

## no\_std

rmodbus supports no\_std mode. Most of the library code is written the way to
support both std and no\_std.

Set dependency as:

```toml
rmodbus = { version = "*", features = ["nostd"] }
```

## Small context

Default Modbus context has 10000 registers of each type, which requires 42500
bytes total. For systems with small RAM amount it's possible to reduce the
context size to the 1000 registers of each type (4250 bytes) with the following
feature:

```toml
rmodbus = { version = "*", features = ["nostd", "smallcontext"] }
```

## Vectors

Some of rmodbus functions use vectors to store result. In std mode, either
standard std::vec::Vec or [FixedVec](https://crates.io/crates/fixedvec) can be
used. In nostd mode, only FixedVec is supported.

## Changelog

### v0.4

* Modbus context is no longer created automatically and no mutex guard is
  provided by default. Use ModbusContext::new() to create context object and
  then use it as you wish - protect with any kind of Mutex, with RwLock or just
  put into UnsafeCell.

* Context SDK changes: all functions moved inside context, removed unnecessary
  ones, function args optimized.

* FixedVec support included by default, both in std and nostd.

* Added support for 64-bit integers

## Modbus client

Planned.
