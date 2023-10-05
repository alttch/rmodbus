use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use lazy_static::lazy_static;

use std::sync::RwLock;

use rmodbus::{
    server::{context::ModbusContextFull, ModbusFrame},
    ModbusFrameBuf, ModbusProto,
};

lazy_static! {
    pub static ref CONTEXT: RwLock<ModbusContextFull> = RwLock::new(ModbusContextFull::new());
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
                        false => frame
                            .process_write(&mut CONTEXT.write().unwrap())
                            .map(|_| ()),
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
