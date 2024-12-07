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

rmodbus is a part of [EVA ICS v4](https://www.eva-ics.com/) industrial
automation stack and [RoboPLC](https://www.roboplc.com/) I/O.

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

For synchronous servers and clients (std) we recommend using [RoboPLC Modbus
I/O](https://docs.rs/roboplc/latest/roboplc/io/modbus/index.html) modules.

Here is an example of a simple TCP blocking server:

```rust
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::sync::RwLock;
use once_cell::sync::Lazy;

use rmodbus::{
    server::{storage::ModbusStorageFull, context::ModbusContext, ModbusFrame},
    ModbusFrameBuf, ModbusProto,
};

static CONTEXT: Lazy<RwLock<ModbusStorageFull>> = Lazy::new(<_>::default);

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
                    let result = if frame.readonly {
                        frame.process_read(&*CONTEXT.read().unwrap())
                    } else {
                        frame.process_write(&mut *CONTEXT.write().unwrap())
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
59Kbytes of RAM.

rmodbus server context is thread-safe, easy to use and has a lot of functions.

The context must be protected with a mutex/rwlock and every time Modbus context
is accessed, a context mutex must be locked. This slows down performance, but
guarantees that the context always has valid data after bulk-sets and after
writes of long data types. So make sure your application locks context only
when required and only for a short period time.

A simple PLC example:

```rust,ignore
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

use rmodbus::server::{storage::ModbusStorageFull, context::ModbusContext};

#[path = "../examples/servers/tcp.rs"]
mod srv;

// put 1 to holding register 1500 to save current context to /tmp/plc1.dat
// if the file exists, context will be loaded at the next start

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

fn save(fname: &str, ctx: &ModbusStorageFull) -> Result<(), Box<dyn Error>> {
    let config = bincode::config::standard();
    let mut file = File::create(fname)?;
    file.write(&bincode::encode_to_vec(ctx, config)?)?;
    file.sync_all()?;
    Ok(())
}

fn load(fname: &str, ctx: &mut ModbusStorageFull) -> Result<(), Box<dyn Error>> {
    let config = bincode::config::standard();
    let mut file = File::open(fname)?;
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data)?;
    (*ctx, _) = bincode::decode_from_slice(&data, config)?;
    Ok(())
}

fn main() {
    // read context
    let unit_id = 1;
    {
        let mut ctx = srv::CONTEXT.write().unwrap();
        let _ = load(&"/tmp/plc1.dat", &mut ctx).map_err(|_| {
            eprintln!("warning: no saved context");
        });
    }
    use std::thread;
    thread::spawn(move || {
        srv::tcpserver(unit_id, "localhost:5502");
    });
    looping();
}
```

To let the above program communicate with outer world, Modbus server must be up
and running in the separate thread, asynchronously or whatever is preferred.

## `no_std`

rmodbus supports `no_std` mode. Most of the library code is written the way to
support both `std` and `no_std`.

For `no_std`, set the dependency as:

```toml
rmodbus = { version = "*", default-features = false }
```

## Small storage

The full Modbus storage has 10000 registers of each type, which requires 60000
bytes total. For systems with small RAM amount there is a pre-defined small
storage with 1000 registers:

```rust
use rmodbus::server::{storage::ModbusStorageSmall, context::ModbusContext};
```

## Custom-sized storage

Starting from the version 0.7 it is allowed to define storage of any size using
generic constants. The generic constants order is: coils, discretes, inputs,
holdings.

E.g. let us define a context for 128 coils, 16 discretes, 0 inputs and 100
holdings:

```rust
use rmodbus::server::{storage::ModbusStorage, context::ModbusContext};

let context = ModbusStorage::<128, 16, 0, 100>::new();
```

## Custom server implementation

Starting from the version 0.9 it is allowed to provide custom server implementation 
by implementing `use rmodbus::server::context::ModbusContext` on custom struct.
For sample implementation have a look at `src/server/storage.rs`

## Custom type representations in `u16` sized registers

Starting from version \<todo: insert version number here\>, you can implement 
`server::RegisterRepresentable<N>` on your own types and use 
`ModbusContext::set_*_as_representable` and `ModbusContext::get_*_as_representable`
methods to directly store and read your own types in the registers.

## Vectors

Some of rmodbus functions use vectors to store result.  Different vector types can be used:

- When the `std` feature is enabled (default), `std::vec::Vec` can be used.
- With the `fixedvec` feature, [`fixedvec::FixedVec`](https://crates.io/crates/fixedvec) can be used.
- With the `heapless` feature, [`heapless::Vec`](https://crates.io/crates/heapless) can be used.

- When the `alloc` feature is enabled, Rust core allocation `alloc::vec::Vec`
  can be used in no-std mode. E.g `cargo build --no-default-features --features
  alloc` builds in no-std mode, and supports using core allocation
  `alloc::vec::Vec`. When `std` feature is enabled, the `alloc` feature is
  ignored.

## Modbus client

Modbus client is designed with the same principles as the server: the crate
gives frame generator / processor, while the frames can be read / written with
any source and with any required way.

TCP client Example:

```rust,no_run
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
## About the authors

[Bohemia Automation](https://www.bohemia-automation.com) /
[Altertech](https://www.altertech.com) is a group of companies with 15+ years
of experience in the enterprise automation and industrial IoT. Our setups
include power plants, factories and urban infrastructure. Largest of them have
1M+ sensors and controlled devices and the bar raises higher and higher every
day.
