use serial::prelude::*;
use std::io::{Read, Write};
use std::time::Duration;

use rmodbus::server::*;

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
                guess_frame_len(&asciibuf, ModbusProto::Ascii).unwrap()
            );
            let mut frame: ModbusFrame = [0; 256];
            let result = parse_ascii_frame(&asciibuf, rd, &mut frame, 0);
            if result.is_err() {
                println!("unable to decode");
                continue;
            } else {
                println!("parsed {} bytes", result.unwrap());
            }
            let mut response = Vec::new();
            let result = process_frame(unit, &frame, ModbusProto::Ascii, &mut response);
            if result.is_err() || response.is_empty() {
                println!("no response to send {:?}", result);
                continue;
            }
            println!("{:x?}", response);
            let mut response_ascii = Vec::new();
            generate_ascii_frame(&response, &mut response_ascii).unwrap();
            println!("{:x?}", response_ascii);
            port.write(response_ascii.as_slice()).unwrap();
        }
    }
}
