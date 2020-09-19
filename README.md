# rmodbus - Modbus for Rust

A framework to build fast and reliable Modbus-powered applications.

Cargo crate: https://crates.io/crates/rmodbus

Usage (default: std, full context, multi-threading):

```toml
[dependencies]
rmodbus = { version = "*", features = ["std"] }
```

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

use rmodbus::server::{ModbusFrame, ModbusProto, process_frame};

pub fn tcpserver(unit: u8, listen: &str) {
    let listener = TcpListener::bind(listen).unwrap();
    println!("listening started, ready to accept");
    for stream in listener.incoming() {
        thread::spawn(move || {
            println!("client connected");
            let mut stream = stream.unwrap();
            loop {
                let mut buf: ModbusFrame = [0; 256];
                let mut response = Vec::new(); // for nostd use FixedVec with alloc [u8;256]
                if stream.read(&mut buf).unwrap_or(0) == 0 {
                    return;
                }
                if process_frame(unit, &buf, ModbusProto::TcpUdp, &mut response).is_err() {
                        println!("server error");
                        return;
                    }
                println!("{:x?}", response.as_slice());
                if !response.is_empty() {
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
use rmodbus::server::context;
use std::fs::File;
use std::io::prelude::*;
use std::sync::MutexGuard;

fn looping() {
    loop {
        // READ WORK MODES ETC
        let mut ctx = context::CONTEXT.lock().unwrap();
        let _param1 = context::get(1000, &ctx.holdings).unwrap();
        let _param2 = context::get_f32(1100, &ctx.holdings).unwrap(); // ieee754 f32
        let _param3 = context::get_u32(1200, &ctx.holdings).unwrap(); // u32
        let cmd = context::get(1500, &ctx.holdings).unwrap();
        context::set(1500, 0, &mut ctx.holdings).unwrap();
        if cmd != 0 {
            println!("got command code {}", cmd);
            match cmd {
                1 => {
                    println!("saving memory context");
                    let _ = save("/tmp/plc1.dat", &mut ctx).map_err(|_| {
                        eprintln!("unable to save context!");
                    });
                }
                _ => println!("command not implemented"),
            }
        }
        drop(ctx);
        // ==============================================
        // DO SOME JOB
        // ..........
        // WRITE RESULTS
        let mut ctx = context::CONTEXT.lock().unwrap();
        context::set(0, true, &mut ctx.coils).unwrap();
        context::set_bulk(10, &(vec![10, 20]), &mut ctx.holdings).unwrap();
        context::set_f32(20, 935.77, &mut ctx.inputs).unwrap();
    }
}

fn save(fname: &str, ctx: &MutexGuard<context::ModbusContext>) -> Result<(), std::io::Error> {
    let mut file = match File::create(fname) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    for i in context::context_iter(&ctx) {
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

## no_std

rmodbus supports no\_std mode. Most of the library code is written the way to
support both std and no\_std.

### Switching library to no_std

Set dependency as:

```toml
rmodbus = { version = "*", features = ["nostd"] }
```

### Types and crates in no\_std mode

* To perform context bulk gets and obtain responses from Modbus frame
  processing, use [FixedVec](https://crates.io/crates/fixedvec) instead of
  std::vec::Vec

* In the no\_std mode, rmodbus context is protected with
  [spin](https://crates.io/crates/spin) Mutex instead of std::sync::mutex. Note
  that spin MutexGuard doesn't require unwrap() after locking.

## Single-threaded and async apps

Single-threaded applications can gain up to +60-100% speed boost by removing
Modbus context mutex. This can be performed by replacing mutex with a fake one.
For the compatibility, the context still need to be "unlocked", however the
fake mutex does this instantly and without any CPU overhead.

```toml
rmodbus = { version = "*", features = ["std", "single"] } # or nostd
```

## Small context

Default Modbus context has 10000 registers of each type, which requires 42500
bytes total. For systems with small RAM amount it's possible to reduce the
context size to the 1000 registers of each type (4250 bytes) with the following
feature:

```toml
rmodbus = { version = "*", features = ["std", "smallcontext"] } # or nostd
```

## Modbus client

Planned.
