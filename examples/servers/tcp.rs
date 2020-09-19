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
