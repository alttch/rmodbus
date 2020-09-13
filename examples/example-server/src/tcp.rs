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
                if stream.read(&mut buf).unwrap_or(0) == 0 {
                    return;
                }
                let response: Vec<u8> = match process_frame(unit, &buf, ModbusProto::TcpUdp) {
                    Some(v) => v,
                    None => {
                        println!("frame drop");
                        continue;
                    }
                };
                println!("{:x?}", response);
                if stream.write(response.as_slice()).is_err() {
                    return;
                }
            }
        });
    }
}
