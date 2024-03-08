use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use rmodbus::{client::ModbusRequest, guess_response_frame_len, ModbusProto};

fn main() {
    let timeout = Duration::from_secs(1);

    // open TCP connection
    let mut stream = TcpStream::connect("localhost:5502").unwrap();
    stream.set_read_timeout(Some(timeout)).unwrap();
    stream.set_write_timeout(Some(timeout)).unwrap();

    // create request object
    let mut mreq = ModbusRequest::new(1, ModbusProto::TcpUdp);
    mreq.tr_id = 2; // just for test, default tr_id is 1

    // set 2 coils
    let mut request = Vec::new();
    mreq.generate_set_coils_bulk(0, &[true, true], &mut request)
        .unwrap();

    // write request to stream
    stream.write_all(&request).unwrap();

    // read first 6 bytes of response frame
    let mut buf = [0u8; 6];
    stream.read_exact(&mut buf).unwrap();
    let mut response = Vec::new();
    response.extend_from_slice(&buf);
    let len = guess_response_frame_len(&buf, ModbusProto::TcpUdp).unwrap();
    // read rest of response frame
    if len > 6 {
        let mut rest = vec![0u8; (len - 6) as usize];
        stream.read_exact(&mut rest).unwrap();
        response.extend(rest);
    }
    // check if frame has no Modbus error inside
    mreq.parse_ok(&response).unwrap();

    // get coil values back
    mreq.generate_get_coils(0, 2, &mut request).unwrap();
    stream.write_all(&request).unwrap();
    let mut buf = [0u8; 6];
    stream.read_exact(&mut buf).unwrap();
    let mut response = Vec::new();
    response.extend_from_slice(&buf);
    let len = guess_response_frame_len(&buf, ModbusProto::TcpUdp).unwrap();
    if len > 6 {
        let mut rest = vec![0u8; (len - 6) as usize];
        stream.read_exact(&mut rest).unwrap();
        response.extend(rest);
    }
    let mut data = Vec::new();
    // check if frame has no Modbus error inside and parse response bools into data vec
    mreq.parse_bool(&response, &mut data).unwrap();
    for (i, c) in data.iter().enumerate() {
        println!("{} {}", i, c);
    }
}
