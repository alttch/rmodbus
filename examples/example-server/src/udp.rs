use std::net::UdpSocket;

use rmodbus::server::{ModbusFrame, ModbusProto, process_frame};

pub fn udpserver(unit: u8, listen: &str) {
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
