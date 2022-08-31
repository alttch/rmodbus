<h2>
  rmodbus - Modbus for Rust
  <a href="https://crates.io/crates/rmodbus"><img alt="crates.io page" src="https://img.shields.io/crates/v/rmodbus.svg"></img></a>
  <a href="https://docs.rs/rmodbus"><img alt="docs.rs page" src="https://docs.rs/rmodbus/badge.svg"></img></a>
  <a href="https://github.com/alttch/rmodbus/actions/workflows/ci.yml">
    <img alt="GitHub Actions CI" src="https://github.com/alttch/rmodbus/actions/workflows/ci.yml/badge.svg"></img>
  </a>
</h2>

A framework to build fast and reliable Modbus-powered applications.

## What is rmodbus

rmodbus is not a yet another Modbus client/server. rmodbus is a set of tools to
quickly build Modbus-powered applications. Consider rmodbus is a
request/response codec, plus context manager.

## Why yet another Modbus lib?

* rmodbus is transport- and protocol-independent
* rmodbus is platform independent (**`no_std` is fully supported!**)
* can be easily used in blocking and async (non-blocking) applications
* tuned for speed and reliability
* provides a set of tools to easily work with Modbus context
* supports client/server frame processing for Modbus TCP/UDP, RTU and ASCII
* server context can be easily managed, imported and exported

## So no server is included?

Yes, there is no server included. You build the server by your own. You choose
the transport protocol, technology and everything else. rmodbus just process
frames and works with Modbus context.

Here is an example of a simple TCP blocking server:

```rust,ignore
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use lazy_static::lazy_static;

use std::sync::RwLock;

use rmodbus::{
    server::{context::ModbusContext, ModbusFrame},
    ModbusFrameBuf, ModbusProto,
};

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

Launch the examples as:

```shell
cargo run --example app
cargo run --example tcpserver
```

## Modbus context

The rule is simple: one standard Modbus context per application. 10k+10k 16-bit
registers and 10k+10k coils are usually more than enough. This takes about
59Kbytes of RAM. You can also opt-out of the `fullcontext` feature, to make the
context 10x more compact (6Kbytes) by reducing number of each register type.

rmodbus server context is thread-safe, easy to use and has a lot of functions.

The context must be protected with a mutex/rwlock and every time Modbus context
is accessed, a context mutex must be locked. This slows down performance, but
guarantees that the context always has valid data after bulk-sets and after
writes of long data types. So make sure your application locks context only
when required and only for a short period time.

A simple PLC example:

```rust,ignore
use std::fs::File;
use std::io::{Write};

use rmodbus::server::context::ModbusContext;

fn looping() {
    println!("Loop started");
    loop {
        // READ WORK MODES ETC
        let ctx = srv::CONTEXT.read().unwrap();
        let _param1 = ctx.get_holding(1000).unwrap();
        let _param2 = ctx.get_holdings_as_f32(1100).unwrap(); // ieee754 f32
        let _param3 = ctx.get_holdings_as_u32(1200).unwrap(); // u32
        let cmd = ctx.get_holding(1500).unwrap();
        drop(ctx);
        if cmd != 0 {
            println!("got command code {}", cmd);
            let mut ctx = srv::CONTEXT.write().unwrap();
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
        let mut ctx = srv::CONTEXT.write().unwrap();
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

## `no_std`

rmodbus supports `no_std` mode. Most of the library code is written the way to
support both `std` and `no_std`.

For `no_std`, set the dependency as:

```toml
rmodbus = { version = "*", default-features = false, features = ["fullcontext"] }
```

## Small context

The full Modbus context has 10000 registers of each type, which requires 60000
bytes total. For systems with small RAM amount it is possible to reduce the
context size to 1000 registers of each type (6000 bytes) by not enabling the
`fullcontext` feature:

```toml
rmodbus = { version = "*", default-features = false }
```

## Vectors

Some of rmodbus functions use vectors to store result.  Different vector types can be used:

- When the `std` feature is enabled (default), `std::vec::Vec` can be used.
- With the `fixedvec` feature, [`fixedvec::FixedVec`](https://crates.io/crates/fixedvec) can be used.
- With the `heapless` feature, [`heapless::Vec`](https://crates.io/crates/fixedvec) can be used.

## Modbus client

Modbus client is designed with the same principles as the server: the crate
gives frame generator / processor, while the frames can be read / written with
any source and with any required way.

TCP client Example:

```rust,ignore
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use rmodbus::{client::ModbusRequest, guess_response_frame_len, ModbusProto};

fn main() {
    let timeout = Duration::from_secs(1);

    // open TCP connection
    let mut stream = TcpStream::connect("localhost:5502").unwrap();
    stream.set_read_timeout(Some(timeout)).unwrap();
    stream.set_write_timeout(Some(timeout)).unwrap();

    // create request object
    let mut mreq = ModbusRequest::new(1, ModbusProto::TcpUdp);
    mreq.tr_id = 2; // just for test, default tr_id is 1

    // set 2 coils
    let mut request = Vec::new();
    mreq.generate_set_coils_bulk(0, &[true, true], &mut request)
        .unwrap();

    // write request to stream
    stream.write(&request).unwrap();

    // read first 6 bytes of response frame
    let mut buf = [0u8; 6];
    stream.read_exact(&mut buf).unwrap();
    let mut response = Vec::new();
    response.extend_from_slice(&buf);
    let len = guess_response_frame_len(&buf, ModbusProto::TcpUdp).unwrap();
    // read rest of response frame
    if len > 6 {
        let mut rest = vec![0u8; (len - 6) as usize];
        stream.read_exact(&mut rest).unwrap();
        response.extend(rest);
    }
    // check if frame has no Modbus error inside
    mreq.parse_ok(&response).unwrap();

    // get coil values back
    mreq.generate_get_coils(0, 2, &mut request).unwrap();
    stream.write(&request).unwrap();
    let mut buf = [0u8; 6];
    stream.read_exact(&mut buf).unwrap();
    let mut response = Vec::new();
    response.extend_from_slice(&buf);
    let len = guess_response_frame_len(&buf, ModbusProto::TcpUdp).unwrap();
    if len > 6 {
        let mut rest = vec![0u8; (len - 6) as usize];
        stream.read_exact(&mut rest).unwrap();
        response.extend(rest);
    }
    let mut data = Vec::new();
    // check if frame has no Modbus error inside and parse response bools into data vec
    mreq.parse_bool(&response, &mut data).unwrap();
    for i in 0..data.len() {
        println!("{} {}", i, data[i]);
    }
}
```
