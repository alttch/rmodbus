use std::net::UdpSocket;

use lazy_static::lazy_static;

use std::sync::RwLock;

use rmodbus::{
    server::{context::ModbusContextFull, ModbusFrame},
    ModbusFrameBuf, ModbusProto,
};

lazy_static! {
    pub static ref CONTEXT: RwLock<ModbusContextFull> = RwLock::new(ModbusContextFull::new());
}

pub fn udpserver(unit: u8, listen: &str) {
    let socket = UdpSocket::bind(listen).unwrap();
    loop {
        let mut buf: ModbusFrameBuf = [0; 256];
        let (_amt, src) = socket.recv_from(&mut buf).unwrap();
        println!("got packet");
        let mut response = Vec::new();
        let mut frame = ModbusFrame::new(unit, &buf, ModbusProto::TcpUdp, &mut response);
        if frame.parse().is_err() {
            println!("server error");
            continue;
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
                continue;
            }
        }
        if frame.response_required {
            frame.finalize_response().unwrap();
            println!("{:x?}", response.as_slice());
            socket.send_to(response.as_slice(), &src).unwrap();
        }
    }
}
