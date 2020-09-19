use serial::prelude::*;
use std::io::{Read, Write};
use std::time::Duration;

use rmodbus::server::{process_frame, ModbusFrame, ModbusProto};

pub fn rtuserver(unit: u8, port: &str) {
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
            let mut response = Vec::new();
            if process_frame(unit, &buf, ModbusProto::Rtu, &mut response).is_err()
                || response.is_empty()
            {
                continue;
            }
            println!("{:x?}", response);
            port.write(response.as_slice()).unwrap();
        }
    }
}
