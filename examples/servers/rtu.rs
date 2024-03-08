use once_cell::sync::Lazy;
use serial::prelude::*;
use std::io::{Read, Write};
use std::sync::RwLock;
use std::time::Duration;

use rmodbus::{
    server::{storage::ModbusStorageFull, ModbusFrame},
    ModbusFrameBuf, ModbusProto,
};

static CONTEXT: Lazy<RwLock<ModbusStorageFull>> = Lazy::new(<_>::default);

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
        let mut buf: ModbusFrameBuf = [0; 256];
        if port.read(&mut buf).unwrap() > 0 {
            println!("got frame");
            let mut response = Vec::new();
            let mut frame = ModbusFrame::new(unit, &buf, ModbusProto::Rtu, &mut response);
            if frame.parse().is_err() {
                println!("server error");
                continue;
            }
            if frame.processing_required {
                let result = match frame.readonly {
                    true => frame.process_read(&*CONTEXT.read().unwrap()),
                    false => frame.process_write(&mut *CONTEXT.write().unwrap()),
                };
                if result.is_err() {
                    println!("frame processing error");
                    continue;
                }
            }
            if frame.response_required {
                frame.finalize_response().unwrap();
                println!("{:x?}", response);
                port.write_all(response.as_slice()).unwrap();
            }
        }
    }
}
