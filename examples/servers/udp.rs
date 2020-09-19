use std::net::UdpSocket;

use rmodbus::server::{ModbusFrame, ModbusProto, process_frame};

pub fn udpserver(unit: u8, listen: &str) {
    let socket = UdpSocket::bind(listen).unwrap();
    loop {
        let mut buf: ModbusFrame = [0; 256];
        let (_amt, src) = socket.recv_from(&mut buf).unwrap();
        println!("got packet");
        let mut response = Vec::new();
        if process_frame(unit, &buf, ModbusProto::TcpUdp, &mut response).is_err() {
                println!("frame drop");
                continue;
        };
        println!("{:x?}", response);
        if !response.is_empty() { socket.send_to(response.as_slice(), &src).unwrap(); }
    }
}
