use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::UdpSocket;
use std::thread;
use serial::prelude::*;
use std::time::Duration;

use rmodbus::server::{ModbusFrame, ModbusProto, process_frame};

fn tcpserver(unit: u8, listen: &str) {
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
                let response: Vec<u8> =
                    match process_frame(unit, &buf, ModbusProto::TcpUdp) {
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

fn udpserver(unit: u8, listen: &str) {
    let socket = UdpSocket::bind(listen).unwrap();
    loop {
        let mut buf: ModbusFrame = [0; 256];
        let (_amt, src) = socket.recv_from(&mut buf).unwrap();
        println!("got packet");
        let response: Vec<u8> = match process_frame(unit, &buf, ModbusProto::TcpUdp) {
            Some(v) => v,
            None => {
                println!("frame drop");
                continue;
            }
        };
        println!("{:x?}", response);
        socket.send_to(response.as_slice(), &src).unwrap();
    }
}

fn rtuserver(unit: u8, port: &str) {
    let mut port = serial::open(port).unwrap();
    port.reconfigure(&|settings| {
        (settings.set_baud_rate(serial::Baud9600).unwrap());
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    })
    .unwrap();
    port.set_timeout(Duration::from_secs(3600)).unwrap();
    loop {
        let mut buf: ModbusFrame = [0; 256];
        if port.read(&mut buf).unwrap() > 0 {
            println!("got frame");
            let response: Vec<u8> = match process_frame(unit, &buf, ModbusProto::Rtu) {
                Some(v) => v,
                None => {
                    println!("frame drop");
                    continue;
                }
            };
            println!("{:x?}", response);
            port.write(response.as_slice()).unwrap();
        }
    }
}

fn main() {
    tcpserver(1, &"127.0.0.1:5502");
    //udpserver(1, &"127.0.0.1:5502");
    //rtuserver(1, "/dev/ttyS0");
}
