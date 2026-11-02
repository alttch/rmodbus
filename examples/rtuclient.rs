use std::io::{Read, Write};
use std::time::Duration;

use serial::prelude::*;

use rmodbus::{client::ModbusRequest, guess_response_frame_len, ModbusProto};

fn main() {
    let mut port = serial::open("/dev/ttyS0").unwrap();
    port.reconfigure(&|s| {
        s.set_baud_rate(serial::Baud9600).unwrap();
        s.set_char_size(serial::Bits8);
        s.set_parity(serial::ParityNone);
        s.set_stop_bits(serial::Stop1);
        s.set_flow_control(serial::FlowNone);
        Ok(())
    })
    .unwrap();
    port.set_timeout(Duration::from_secs(1)).unwrap();

    let mut mreq = ModbusRequest::new(1, ModbusProto::Rtu);

    let mut request = Vec::new();
    mreq.generate_get_holdings(0, 2, &mut request).unwrap();
    port.write_all(&request).unwrap();

    // guess_response_frame_len needs 3 bytes for a normal response but only 2 for an
    // exception, so peek at the function byte before deciding how much more to read.
    let mut response = vec![0u8; 2];
    port.read_exact(&mut response).unwrap();

    let total_len = if response[1] & 0x80 != 0 {
        5
    } else {
        response.resize(3, 0);
        port.read_exact(&mut response[2..]).unwrap();
        usize::from(guess_response_frame_len(&response, ModbusProto::Rtu).unwrap())
    };

    if response.len() < total_len {
        let already_read = response.len();
        response.resize(total_len, 0);
        port.read_exact(&mut response[already_read..]).unwrap();
    }

    let mut values = Vec::new();
    match mreq.parse_u16(&response, &mut values) {
        Ok(()) => {
            for (i, v) in values.iter().enumerate() {
                println!("{} {}", i, v);
            }
        }
        Err(e) => println!("modbus error: {}", e),
    }
}
