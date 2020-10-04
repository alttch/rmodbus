use serial::prelude::*;
use std::io::{Read, Write};
use std::time::Duration;

use lazy_static::lazy_static;

use std::sync::RwLock;

use rmodbus::{
    generate_ascii_frame, guess_request_frame_len, parse_ascii_frame,
    server::{context::ModbusContext, ModbusFrame},
    ModbusFrameBuf, ModbusProto,
};

lazy_static! {
    pub static ref CONTEXT: RwLock<ModbusContext> = RwLock::new(ModbusContext::new());
}

pub fn asciiserver(unit: u8, port: &str) {
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
        let mut asciibuf = [0; 1024];
        let rd = port.read(&mut asciibuf).unwrap();
        if rd > 0 {
            println!("got frame len {}", rd);
            println!(
                "{}",
                guess_request_frame_len(&asciibuf, ModbusProto::Ascii).unwrap()
            );
            let mut buf: ModbusFrameBuf = [0; 256];
            let result = parse_ascii_frame(&asciibuf, rd, &mut buf, 0);
            if result.is_err() {
                println!("unable to decode");
                continue;
            } else {
                println!("parsed {} bytes", result.unwrap());
            }
            let mut response = Vec::new();
            let mut frame = ModbusFrame::new(unit, &buf, ModbusProto::Ascii, &mut response);
            if frame.parse().is_err() {
                println!("server error");
                continue;
            }
            if frame.processing_required {
                let result = match frame.readonly {
                    true => frame.process_read(&CONTEXT.read().unwrap()),
                    false => frame.process_write(&mut CONTEXT.write().unwrap()),
                };
                if result.is_err() {
                    println!("frame processing error");
                    continue;
                }
            }
            if frame.response_required {
                frame.finalize_response().unwrap();
                println!("{:x?}", response);
                let mut response_ascii = Vec::new();
                generate_ascii_frame(&response, &mut response_ascii).unwrap();
                println!("{:x?}", response_ascii);
                port.write(response_ascii.as_slice()).unwrap();
            }
        }
    }
}
