use crate::client::*;
use crate::server::context::ModbusContext;
use crate::server::storage::{ModbusStorageFull, FULL_STORAGE_SIZE as STORAGE_SIZE};
use crate::server::*;
use crate::*;
use crc16::*;
use std::sync::RwLock;

lazy_static! {
    pub static ref CTX: RwLock<ModbusStorageFull> = RwLock::new(ModbusStorageFull::new());
}

#[test]
fn test_std_read_coils_as_bytes_oob() {
    let mut ctx = CTX.write().unwrap();
    let mut result = Vec::new();
    match ctx.get_coils_bulk(0, STORAGE_SIZE as u16 + 1, &mut result) {
        Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
        Err(_) => assert!(true),
    }
    match ctx.get_coils_bulk(STORAGE_SIZE as u16, 1, &mut result) {
        Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
        Err(_) => assert!(true),
    }
    ctx.get_coils_bulk((STORAGE_SIZE - 1) as u16, 1, &mut result)
        .unwrap();
    match ctx.get_coils_bulk(STORAGE_SIZE as u16 - 1, 2, &mut result) {
        Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
        Err(_) => assert!(true),
    }
    ctx.get_coil((STORAGE_SIZE - 1) as u16).unwrap();
    match ctx.get_coil(STORAGE_SIZE as u16) {
        Ok(_) => assert!(false, "oob failed MAX"),
        Err(_) => assert!(true),
    }
    ctx.set_coil((STORAGE_SIZE - 1) as u16, true).unwrap();
    match ctx.set_coil(STORAGE_SIZE as u16, true) {
        Ok(_) => assert!(false, "oob failed MAX"),
        Err(_) => assert!(true),
    }
}

#[test]
fn test_std_coil_get_set_bulk() {
    let mut ctx = CTX.write().unwrap();
    let mut data = Vec::new();
    let mut result = Vec::new();
    data.extend_from_slice(&[true; 2]);
    ctx.set_coils_bulk(5, &data.as_slice()).unwrap();
    ctx.get_coils_bulk(5, 2, &mut result).unwrap();
    assert_eq!(result, data);

    data.clear();
    result.clear();

    data.extend_from_slice(&[true; 18]);
    ctx.set_coils_bulk(25, &data.as_slice()).unwrap();
    ctx.get_coils_bulk(25, 18, &mut result).unwrap();
    assert_eq!(result, data);

    ctx.set_coil(28, true).unwrap();
    assert_eq!(ctx.get_coil(28).unwrap(), true);
    ctx.set_coil(28, false).unwrap();
    assert_eq!(ctx.get_coil(28).unwrap(), false);
}

#[test]
fn test_std_read_discretes_as_bytes_oob() {
    let mut ctx = CTX.write().unwrap();
    let mut result = Vec::new();
    match ctx.get_discretes_bulk(0, STORAGE_SIZE as u16 + 1, &mut result) {
        Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
        Err(_) => assert!(true),
    }
    match ctx.get_discretes_bulk(STORAGE_SIZE as u16, 1, &mut result) {
        Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
        Err(_) => assert!(true),
    }
    ctx.get_discretes_bulk((STORAGE_SIZE - 1) as u16, 1, &mut result)
        .unwrap();
    match ctx.get_discretes_bulk(STORAGE_SIZE as u16 - 1, 2, &mut result) {
        Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
        Err(_) => assert!(true),
    }
    ctx.get_discrete((STORAGE_SIZE - 1) as u16).unwrap();
    match ctx.get_discrete(STORAGE_SIZE as u16) {
        Ok(_) => assert!(false, "oob failed MAX"),
        Err(_) => assert!(true),
    }
    ctx.set_discrete((STORAGE_SIZE - 1) as u16, true).unwrap();
    match ctx.set_discrete(STORAGE_SIZE as u16, true) {
        Ok(_) => assert!(false, "oob failed MAX"),
        Err(_) => assert!(true),
    }
}

#[test]
fn test_std_discrete_get_set_bulk() {
    let mut ctx = CTX.write().unwrap();
    let mut data = Vec::new();
    let mut result = Vec::new();
    data.extend_from_slice(&[true; 2]);
    ctx.set_discretes_bulk(5, &data.as_slice()).unwrap();
    ctx.get_discretes_bulk(5, 2, &mut result).unwrap();
    assert_eq!(result, data);

    data.clear();
    result.clear();

    data.extend_from_slice(&[true; 18]);
    ctx.set_discretes_bulk(25, &data.as_slice()).unwrap();
    ctx.get_discretes_bulk(25, 18, &mut result).unwrap();
    assert_eq!(result, data);

    ctx.set_discrete(28, true).unwrap();
    assert_eq!(ctx.get_discrete(28).unwrap(), true);
    ctx.set_discrete(28, false).unwrap();
    assert_eq!(ctx.get_discrete(28).unwrap(), false);
}

#[test]
fn test_std_read_inputs_as_bytes_oob() {
    let mut ctx = CTX.write().unwrap();
    let mut result = Vec::new();
    match ctx.get_inputs_bulk(0, STORAGE_SIZE as u16 + 1, &mut result) {
        Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
        Err(_) => assert!(true),
    }
    match ctx.get_inputs_bulk(STORAGE_SIZE as u16, 1, &mut result) {
        Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
        Err(_) => assert!(true),
    }
    ctx.get_inputs_bulk((STORAGE_SIZE - 1) as u16, 1, &mut result)
        .unwrap();
    match ctx.get_inputs_bulk(STORAGE_SIZE as u16 - 1, 2, &mut result) {
        Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
        Err(_) => assert!(true),
    }
    ctx.get_input((STORAGE_SIZE - 1) as u16).unwrap();
    match ctx.get_input(STORAGE_SIZE as u16) {
        Ok(_) => assert!(false, "oob failed MAX"),
        Err(_) => assert!(true),
    }
    ctx.set_input((STORAGE_SIZE - 1) as u16, 0x55).unwrap();
    match ctx.set_input(STORAGE_SIZE as u16, 0x55) {
        Ok(_) => assert!(false, "oob failed MAX"),
        Err(_) => assert!(true),
    }
    match ctx.set_inputs_from_u32((STORAGE_SIZE - 1) as u16, 0x55) {
        Ok(_) => assert!(false, "oob failed MAX u32"),
        Err(_) => assert!(true),
    }
    ctx.set_inputs_from_u32((STORAGE_SIZE - 2) as u16, 0x9999)
        .unwrap();
    match ctx.set_inputs_from_u64((STORAGE_SIZE - 3) as u16, 0x55) {
        Ok(_) => assert!(false, "oob failed MAX u64"),
        Err(_) => assert!(true),
    }
    ctx.set_inputs_from_u64((STORAGE_SIZE - 4) as u16, 0x9999)
        .unwrap();
}

#[test]
fn test_std_get_set_inputs() {
    let mut ctx = CTX.write().unwrap();
    let mut data = Vec::new();
    let mut result = Vec::new();

    ctx.clear_inputs();

    data.extend_from_slice(&[0x77; 2]);
    ctx.set_inputs_bulk(5, &data.as_slice()).unwrap();
    ctx.get_inputs_bulk(5, 2, &mut result).unwrap();
    assert_eq!(result, data);

    data.clear();
    result.clear();

    data.extend_from_slice(&[0x33; 18]);
    ctx.set_inputs_bulk(25, &data.as_slice()).unwrap();
    ctx.get_inputs_bulk(25, 18, &mut result).unwrap();
    assert_eq!(result, data);

    ctx.set_input(28, 99).unwrap();
    assert_eq!(ctx.get_input(28).unwrap(), 99);
    ctx.set_input(28, 95).unwrap();
    assert_eq!(ctx.get_input(28).unwrap(), 95);
    ctx.set_inputs_from_u32(100, 1234567).unwrap();
    assert_eq!(ctx.get_inputs_as_u32(100).unwrap(), 1234567);
    ctx.set_inputs_from_u64(90, 18_446_744_073_709_551_615)
        .unwrap();
    assert_eq!(
        ctx.get_inputs_as_u64(90).unwrap(),
        18_446_744_073_709_551_615
    );
    ctx.set_inputs_from_f32(200, 1234.567).unwrap();
    assert_eq!(ctx.get_inputs_as_f32(200).unwrap(), 1234.567);
}

#[test]
fn test_std_read_holdings_as_bytes_oob() {
    let mut ctx = CTX.write().unwrap();
    let mut result = Vec::new();
    match ctx.get_holdings_bulk(0, STORAGE_SIZE as u16 + 1, &mut result) {
        Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
        Err(_) => assert!(true),
    }
    match ctx.get_holdings_bulk(STORAGE_SIZE as u16, 1, &mut result) {
        Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
        Err(_) => assert!(true),
    }
    ctx.get_holdings_bulk((STORAGE_SIZE - 1) as u16, 1, &mut result)
        .unwrap();
    match ctx.get_holdings_bulk(STORAGE_SIZE as u16 - 1, 2, &mut result) {
        Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
        Err(_) => assert!(true),
    }
    ctx.get_holding((STORAGE_SIZE - 1) as u16).unwrap();
    match ctx.get_holding(STORAGE_SIZE as u16) {
        Ok(_) => assert!(false, "oob failed MAX"),
        Err(_) => assert!(true),
    }
    ctx.set_holding((STORAGE_SIZE - 1) as u16, 0x55).unwrap();
    match ctx.set_holding(STORAGE_SIZE as u16, 0x55) {
        Ok(_) => assert!(false, "oob failed MAX"),
        Err(_) => assert!(true),
    }
    match ctx.set_holdings_from_u32((STORAGE_SIZE - 1) as u16, 0x55) {
        Ok(_) => assert!(false, "oob failed MAX u32"),
        Err(_) => assert!(true),
    }
    ctx.set_holdings_from_u32((STORAGE_SIZE - 2) as u16, 0x9999)
        .unwrap();
    match ctx.set_holdings_from_u64((STORAGE_SIZE - 3) as u16, 0x55) {
        Ok(_) => assert!(false, "oob failed MAX u64"),
        Err(_) => assert!(true),
    }
    ctx.set_holdings_from_u64((STORAGE_SIZE - 4) as u16, 0x9999)
        .unwrap();
}

#[test]
fn test_std_get_set_holdings() {
    let mut ctx = CTX.write().unwrap();
    let mut data = Vec::new();
    let mut result = Vec::new();

    ctx.clear_holdings();

    data.extend_from_slice(&[0x77; 2]);
    ctx.set_holdings_bulk(5, &data.as_slice()).unwrap();
    ctx.get_holdings_bulk(5, 2, &mut result).unwrap();
    assert_eq!(result, data);

    data.clear();
    result.clear();

    data.extend_from_slice(&[0x33; 18]);
    ctx.set_holdings_bulk(25, &data.as_slice()).unwrap();
    ctx.get_holdings_bulk(25, 18, &mut result).unwrap();
    assert_eq!(result, data);

    ctx.set_holding(28, 99).unwrap();
    assert_eq!(ctx.get_holding(28).unwrap(), 99);
    ctx.set_holding(28, 95).unwrap();
    assert_eq!(ctx.get_holding(28).unwrap(), 95);
    ctx.set_holdings_from_u32(100, 1234567).unwrap();
    assert_eq!(ctx.get_holdings_as_u32(100).unwrap(), 1234567);
    ctx.set_holdings_from_u64(90, 18_446_744_073_709_551_615)
        .unwrap();
    assert_eq!(
        ctx.get_holdings_as_u64(90).unwrap(),
        18_446_744_073_709_551_615
    );
    ctx.set_holdings_from_f32(200, 1234.567).unwrap();
    assert_eq!(ctx.get_holdings_as_f32(200).unwrap(), 1234.567);
}

#[test]
fn test_std_get_bools_as_u8() {
    let mut data = Vec::new();
    let mut ctx = CTX.write().unwrap();
    ctx.clear_coils();
    data.extend_from_slice(&[true, true, true, true, true, true, false, false]);
    ctx.set_coils_bulk(0, &data.as_slice()).unwrap();
    let mut result = Vec::new();
    ctx.get_coils_as_u8(0, 6, &mut result).unwrap();
    assert_eq!(*result.get(0).unwrap(), 0b00111111);
    result.clear();
    ctx.get_coils_as_u8(0, 5, &mut result).unwrap();
    assert_eq!(*result.get(0).unwrap(), 0b00011111);
    result.clear();

    data.clear();
    data.extend_from_slice(&[true, true, false, true, true, true, true, true]);
    ctx.set_coils_bulk(0, &data.as_slice()).unwrap();
    let mut result = Vec::new();
    ctx.get_coils_as_u8(0, 6, &mut result).unwrap();
    assert_eq!(*result.get(0).unwrap(), 0b00111011);
    result.clear();
    ctx.get_coils_as_u8(0, 5, &mut result).unwrap();
    assert_eq!(*result.get(0).unwrap(), 0b00011011);
    result.clear();

    data.clear();
    data.extend_from_slice(&[
        true, true, false, true, true, true, true, true, // byte 1
        true, true, true, true, false, false, true, false, // byte 2
        false, false, false, true, false, true, // byte 3
    ]);
    ctx.set_coils_bulk(0, &data.as_slice()).unwrap();
    let mut result = Vec::new();
    ctx.get_coils_as_u8(0, 22, &mut result).unwrap();
    assert_eq!(*result.get(0).unwrap(), 0b11111011);
    assert_eq!(*result.get(1).unwrap(), 0b01001111);
    assert_eq!(*result.get(2).unwrap(), 0b101000);
}

#[test]
fn test_std_get_set_regs_as_u8() {
    let mut data = Vec::new();
    let mut ctx = CTX.write().unwrap();
    data.extend_from_slice(&[2, 45, 4559, 31, 394, 1, 9, 7, 0, 1, 9]);
    ctx.clear_holdings();
    ctx.set_holdings_bulk(0, &data.as_slice()).unwrap();
    let mut result = Vec::new();
    ctx.get_holdings_as_u8(0, data.len() as u16, &mut result)
        .unwrap();
    assert_eq!(result[0], 0);
    assert_eq!(result[1], 2);
    for i in 0..10 {
        ctx.set_holding(i, 0).unwrap();
    }
    ctx.set_holdings_from_u8(0, &result.as_slice()).unwrap();
    let mut result = Vec::new();
    ctx.get_holdings_bulk(0, data.len() as u16, &mut result)
        .unwrap();
    assert_eq!(result, data);
}

#[test]
fn test_std_get_set_bools_as_u8() {
    let mut ctx = CTX.write().unwrap();
    ctx.clear_coils();
    let mut data = Vec::new();
    data.extend_from_slice(&[
        true, true, true, false, true, true, true, true, true, false, false, false, false, false,
    ]);
    ctx.set_coils_bulk(0, &data.as_slice()).unwrap();
    ctx.set_coil(data.len() as u16, true).unwrap();
    ctx.set_coil(data.len() as u16 + 1, false).unwrap();
    ctx.set_coil(data.len() as u16 + 2, true).unwrap();
    let mut result = Vec::new();
    ctx.get_coils_as_u8(0, data.len() as u16, &mut result)
        .unwrap();
    ctx.set_coils_from_u8(0, data.len() as u16, &result.as_slice())
        .unwrap();
    let mut result = Vec::new();
    ctx.get_coils_bulk(0, data.len() as u16, &mut result)
        .unwrap();
    assert_eq!(result, data);
    result.clear();
    data.push(true);
    data.push(false);
    data.push(true);
    ctx.get_coils_bulk(0, data.len() as u16, &mut result)
        .unwrap();
    assert_eq!(result, data);
}

fn gen_tcp_frame(data: &[u8]) -> ModbusFrameBuf {
    let mut frame: ModbusFrameBuf = [0; 256];
    frame[0] = 0x77;
    frame[1] = 0x55;
    frame[2] = 0;
    frame[3] = 0;
    let len = (data.len() as u16).to_be_bytes();
    frame[4] = len[0];
    frame[5] = len[1];
    for (i, v) in data.iter().enumerate() {
        frame[i + 6] = *v;
    }
    assert_eq!(
        guess_request_frame_len(&frame, ModbusProto::TcpUdp).unwrap(),
        (data.len() + 6) as u8
    );
    frame
}

// also automatically checks server::guest_rtu_frame_len
fn gen_rtu_frame(data: &[u8]) -> ModbusFrameBuf {
    let mut frame: ModbusFrameBuf = [0; 256];
    for (i, v) in data.iter().enumerate() {
        frame[i] = *v;
    }
    let len = data.len();
    let crc16 = State::<MODBUS>::calculate(data);
    let c = crc16.to_le_bytes();
    frame[len] = c[0];
    frame[len + 1] = c[1];
    assert_eq!(
        guess_request_frame_len(&frame, ModbusProto::Rtu).unwrap(),
        (len + 2) as u8
    );
    frame
}

fn check_rtu_response(result: &Vec<u8>, response: &[u8]) {
    let mut resp = Vec::new();
    let mut r = Vec::new();
    for i in 6..response.len() {
        resp.push(response[i]);
    }
    for i in 0..result.len() - 2 {
        r.push(result[i]);
    }
    assert_eq!(resp, r);
    resp.insert(0, 1);
    let result_crc = u16::from_le_bytes([result[result.len() - 2], result[result.len() - 1]]);
    assert_eq!(result_crc, State::<MODBUS>::calculate(r.as_slice()));
}

#[test]
fn test_std_frame_fc01_fc02_fc03_fc04_unknown_function() {
    let mut ctx = CTX.write().unwrap();
    ctx.clear_all();
    let mut result = Vec::new();

    // read coils
    ctx.set_coil(5, true).unwrap();
    ctx.set_coil(7, true).unwrap();
    ctx.set_coil(9, true).unwrap();
    let request = [1, 1, 0, 5, 0, 5];
    let response = [0x77, 0x55, 0, 0, 0, 4, 1, 1, 1, 0x15];
    let framebuf = gen_tcp_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.readonly, true);
    assert_eq!(frame.count, 5);
    assert_eq!(frame.reg, 5);
    assert_eq!(frame.error, 0);
    frame.process_read(&*ctx).unwrap();
    assert_eq!(frame.error, 0);
    frame.finalize_response().unwrap();
    assert_eq!(result.as_slice(), response);

    let mut framebuf = gen_rtu_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.readonly, true);
    assert_eq!(frame.count, 5);
    assert_eq!(frame.reg, 5);
    assert_eq!(frame.error, 0);
    frame.process_read(&*ctx).unwrap();
    assert_eq!(frame.error, 0);
    frame.finalize_response().unwrap();
    check_rtu_response(&result, &response);
    // check rtu crc error
    framebuf[request.len() + 1] = ((framebuf[request.len() + 1] as u16) + 1) as u8;
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
    match frame.parse() {
        Ok(_) => panic!(),
        Err(e) => match e {
            ErrorKind::FrameCRCError => {}
            _ => panic!(),
        },
    }
    // check illegal_function
    let request = [1, 7, 0x27, 0xe, 0, 0xf];
    let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x87, 1];
    let framebuf = gen_tcp_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, false);
    assert_eq!(frame.error, 1);
    frame.finalize_response().unwrap();
    assert_eq!(result.as_slice(), response);

    let framebuf = gen_rtu_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, false);
    assert_eq!(frame.error, 1);
    frame.finalize_response().unwrap();
    check_rtu_response(&result, &response);
    // check context oob
    let request = [1, 1, 0x27, 0xe, 0, 0xf];
    let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x81, 2];
    let framebuf = gen_tcp_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    frame.process_read(&*ctx).unwrap();
    assert_eq!(frame.error, 2);
    frame.finalize_response().unwrap();
    assert_eq!(result.as_slice(), response);
    let framebuf = gen_rtu_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    frame.process_read(&*ctx).unwrap();
    assert_eq!(frame.error, 2);
    frame.finalize_response().unwrap();
    check_rtu_response(&result, &response);
    // check invalid length
    let request = [1, 1, 0, 5, 0, 5];
    let mut framebuf = gen_tcp_frame(&request);
    framebuf[5] = 2;
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    match frame.parse() {
        Ok(_) => panic!(),
        Err(e) => match e {
            ErrorKind::FrameBroken => {}
            _ => panic!("{:?}", e),
        },
    }
    let mut framebuf = gen_tcp_frame(&request);
    framebuf[5] = 251;
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    match frame.parse() {
        Ok(_) => panic!(),
        Err(e) => match e {
            ErrorKind::FrameBroken => {}
            _ => panic!("{:?}", e),
        },
    }
    let mut framebuf = gen_tcp_frame(&request);
    framebuf[3] = 22;
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    match frame.parse() {
        Ok(_) => panic!(),
        Err(e) => match e {
            ErrorKind::FrameBroken => {}
            _ => panic!("{:?}", e),
        },
    }
    // read discretes
    ctx.set_discrete(10, true).unwrap();
    ctx.set_discrete(12, true).unwrap();
    ctx.set_discrete(16, true).unwrap();
    let framebuf = gen_tcp_frame(&[1, 2, 0, 5, 0, 0x10]);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, true);
    frame.process_read(&*ctx).unwrap();
    frame.finalize_response().unwrap();
    assert_eq!(
        result.as_slice(),
        [0x77, 0x55, 0, 0, 0, 5, 1, 2, 2, 0xa0, 8]
    );
    // read holdings
    ctx.set_holding(2, 9977).unwrap();
    ctx.set_holding(4, 9543).unwrap();
    ctx.set_holding(7, 9522).unwrap();
    let request = [1, 3, 0, 0, 0, 0xb];
    let framebuf = gen_tcp_frame(&request);
    let response = [
        0x77, 0x55, 0, 0, 0, 0x19, 1, 3, 0x16, 0, 0, 0, 0, 0x26, 0xf9, 0, 0, 0x25, 0x47, 0, 0, 0,
        0, 0x25, 0x32, 0, 0, 0, 0, 0, 0,
    ];
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, true);
    frame.process_read(&*ctx).unwrap();
    frame.finalize_response().unwrap();
    assert_eq!(result.as_slice(), response);
    let framebuf = gen_rtu_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, true);
    frame.process_read(&*ctx).unwrap();
    frame.finalize_response().unwrap();
    check_rtu_response(&result, &response);
    // read inputs
    ctx.set_input(280, 99).unwrap();
    ctx.set_input(281, 15923).unwrap();
    ctx.set_input(284, 54321).unwrap();
    let framebuf = gen_tcp_frame(&[1, 4, 1, 0x18, 0, 6]);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, true);
    frame.process_read(&*ctx).unwrap();
    frame.finalize_response().unwrap();
    assert_eq!(
        result.as_slice(),
        [0x77, 0x55, 0, 0, 0, 0xf, 1, 4, 0xc, 0, 0x63, 0x3e, 0x33, 0, 0, 0, 0, 0xd4, 0x31, 0, 0]
    );
}

#[test]
fn test_std_frame_fc05_fc06() {
    let mut ctx = CTX.write().unwrap();
    ctx.clear_all();
    let mut result = Vec::new();
    // write coil
    let request = [1, 5, 0, 0xb, 0xff, 0];
    let response = [0x77, 0x55, 0, 0, 0, 6, 1, 5, 0, 0xb, 0xff, 0];
    let framebuf = gen_tcp_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 0);
    frame.finalize_response().unwrap();
    assert_eq!(result.as_slice(), response);
    assert_eq!(ctx.get_coil(11).unwrap(), true);
    let framebuf = gen_rtu_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 0);
    frame.finalize_response().unwrap();
    check_rtu_response(&result, &response);
    // write coil broadcast tcp
    let request = [0, 5, 0, 0x5, 0xff, 0];
    let framebuf = gen_tcp_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, false);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 0);
    assert_eq!(ctx.get_coil(5).unwrap(), true);
    let request = [0, 5, 0, 0x7, 0xff, 0];
    let framebuf = gen_rtu_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, false);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 0);
    assert_eq!(ctx.get_coil(7).unwrap(), true);
    // write coil invalid data
    let request = [1, 5, 0, 0xb, 0xff, 1];
    let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x85, 3];
    let framebuf = gen_tcp_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 3);
    frame.finalize_response().unwrap();
    assert_eq!(result.as_slice(), response);
    // write coil context oob
    let request = [1, 5, 0x99, 0x99, 0xff, 0];
    let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x85, 2];
    let framebuf = gen_tcp_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 2);
    frame.finalize_response().unwrap();
    assert_eq!(result.as_slice(), response);
    let framebuf = gen_rtu_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    frame.finalize_response().unwrap();
    check_rtu_response(&result, &response);
    // write holding
    let request = [1, 6, 0, 0xc, 0x33, 0x55];
    let response = [0x77, 0x55, 0, 0, 0, 6, 1, 6, 0, 0xc, 0x33, 0x55];
    let framebuf = gen_tcp_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 0);
    frame.finalize_response().unwrap();
    assert_eq!(result.as_slice(), response);
    assert_eq!(ctx.get_holding(12).unwrap(), 0x3355);
    let framebuf = gen_rtu_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 0);
    frame.finalize_response().unwrap();
    check_rtu_response(&result, &response);
    // write holding context oob
    let request = [1, 6, 0xff, 0xc, 0x33, 0x55];
    let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x86, 2];
    let framebuf = gen_tcp_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 2);
    frame.finalize_response().unwrap();
    assert_eq!(result.as_slice(), response);
    let framebuf = gen_rtu_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 2);
    frame.finalize_response().unwrap();
    check_rtu_response(&result, &response);
}

#[test]
fn test_std_frame_fc15() {
    let mut ctx = CTX.write().unwrap();
    ctx.clear_all();
    let mut result = Vec::new();
    // write multiple coils
    let request = [1, 0xf, 1, 0x31, 0, 5, 1, 0x25]; // 6 bits in data but 5 coils
    let response = [0x77, 0x55, 0, 0, 0, 6, 1, 0xf, 01, 0x31, 0, 5];
    let framebuf = gen_tcp_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 0);
    frame.finalize_response().unwrap();
    assert_eq!(result.as_slice(), response);
    assert_eq!(ctx.get_coil(305).unwrap(), true);
    assert_eq!(ctx.get_coil(306).unwrap(), false);
    assert_eq!(ctx.get_coil(307).unwrap(), true);
    assert_eq!(ctx.get_coil(308).unwrap(), false);
    assert_eq!(ctx.get_coil(309).unwrap(), false);
    assert_eq!(ctx.get_coil(310).unwrap(), false);
    let framebuf = gen_rtu_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 0);
    frame.finalize_response().unwrap();
    check_rtu_response(&result, &response);
    // write coils context oob
    let request = [1, 0xf, 0x99, 0xe8, 0, 5, 1, 0x25]; // 6 bits in data but 5 coils
    let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x8f, 2];
    let framebuf = gen_tcp_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 2);
    frame.finalize_response().unwrap();
    assert_eq!(result.as_slice(), response);
    let framebuf = gen_rtu_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 2);
    frame.finalize_response().unwrap();
    check_rtu_response(&result, &response);
}

#[test]
fn test_std_frame_fc16() {
    let mut ctx = CTX.write().unwrap();
    ctx.clear_all();
    let mut result = Vec::new();
    // write multiple holdings
    let request = [
        1, 0x10, 1, 0x2c, 0, 4, 8, 0x11, 0x22, 0x11, 0x33, 0x11, 0x55, 0x11, 0x99,
    ];
    let response = [0x77, 0x55, 0, 0, 0, 6, 1, 0x10, 1, 0x2c, 0, 4];
    let framebuf = gen_tcp_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.func, 0x10);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 0);
    frame.finalize_response().unwrap();
    assert_eq!(result.as_slice(), response);
    assert_eq!(ctx.get_holding(300).unwrap(), 0x1122);
    assert_eq!(ctx.get_holding(301).unwrap(), 0x1133);
    assert_eq!(ctx.get_holding(302).unwrap(), 0x1155);
    assert_eq!(ctx.get_holding(303).unwrap(), 0x1199);
    let framebuf = gen_rtu_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 0);
    frame.finalize_response().unwrap();
    check_rtu_response(&result, &response);
    // write holdings context oob
    let request = [
        1, 0x10, 0x99, 0xe8, 0, 4, 8, 0x11, 0x22, 0x11, 0x33, 0x11, 0x55, 0x11, 0x99,
    ];
    let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x90, 2];
    let framebuf = gen_tcp_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 2);
    frame.finalize_response().unwrap();
    assert_eq!(result.as_slice(), response);
    let framebuf = gen_rtu_frame(&request);
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Rtu, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, false);
    frame.process_write(&mut *ctx).unwrap();
    assert_eq!(frame.error, 2);
    frame.finalize_response().unwrap();
    check_rtu_response(&result, &response);
}

#[test]
fn test_modbus_ascii() {
    let ctx = ModbusStorageFull::new();
    let mut result = Vec::new();
    let mut ascii_result = Vec::new();
    let request = [
        0x3a, 0x30, 0x31, 0x30, 0x33, 0x30, 0x30, 0x30, 0x32, 0x30, 0x30, 0x30, 0x31, 0x46, 0x39,
        0xd, 0xa,
    ];
    let response = [
        0x3a, 0x30, 0x31, 0x30, 0x33, 0x30, 0x32, 0x30, 0x30, 0x30, 0x30, 0x46, 0x41, 0xd, 0xa,
    ];
    let mut framebuf: ModbusFrameBuf = [0; 256];
    parse_ascii_frame(&request, request.len(), &mut framebuf, 0).unwrap();
    let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::Ascii, &mut result);
    frame.parse().unwrap();
    assert_eq!(frame.response_required, true);
    assert_eq!(frame.processing_required, true);
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, true);
    frame.process_read(&ctx).unwrap();
    assert_eq!(frame.error, 0);
    frame.finalize_response().unwrap();
    generate_ascii_frame(&result.as_slice(), &mut ascii_result).unwrap();
    assert_eq!(ascii_result.as_slice(), response);
}

#[test]
fn test_std_client() {
    let mut ctx = CTX.write().unwrap();
    ctx.clear_discretes();
    let coils = [
        true, true, true, false, true, true, false, true, true, false, true,
    ];
    let holdings = [2345u16, 4723, 193, 3845, 8321, 1244, 8723, 2231, 48572];
    let holdstr = "The Quick Brown Fox Jumps Over The Lazy Dog";
    let protos = [ModbusProto::TcpUdp, ModbusProto::Rtu, ModbusProto::Ascii];

    for proto in &protos {
        // set coils bulk
        ctx.clear_coils();
        let mut mreq = ModbusRequest::new(2, *proto);
        let mut request = Vec::new();
        mreq.generate_set_coils_bulk(100, &coils, &mut request)
            .unwrap();
        let mut response = Vec::new();
        let mut framebuf: ModbusFrameBuf = [0; 256];
        if *proto == ModbusProto::Rtu {
            let mut ascii_frame = Vec::new();
            generate_ascii_frame(&request, &mut ascii_frame).unwrap();
            for i in 0..framebuf.len() {
                framebuf[i] = 0
            }
            parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
        } else {
            for i in 0..request.len() {
                framebuf[i] = request[i];
            }
        }
        let mut frame = ModbusFrame::new(2, &framebuf, *proto, &mut response);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut *ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        mreq.parse_ok(&response).unwrap();
        for i in 100..100 + coils.len() {
            assert_eq!(ctx.get_coil(i as u16).unwrap(), coils[i - 100]);
        }

        // reading coils
        let mut mreq = ModbusRequest::new(3, *proto);
        let mut request = Vec::new();
        mreq.generate_get_coils(100, coils.len() as u16, &mut request)
            .unwrap();
        let mut response = Vec::new();
        let mut framebuf: ModbusFrameBuf = [0; 256];
        if *proto == ModbusProto::Rtu {
            let mut ascii_frame = Vec::new();
            generate_ascii_frame(&request, &mut ascii_frame).unwrap();
            for i in 0..framebuf.len() {
                framebuf[i] = 0
            }
            parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
        } else {
            for i in 0..request.len() {
                framebuf[i] = request[i];
            }
        }
        let mut frame = ModbusFrame::new(3, &framebuf, *proto, &mut response);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, true);
        frame.process_read(&mut *ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        let mut result = Vec::new();
        mreq.parse_bool(&response, &mut result).unwrap();
        assert_eq!(result, coils);

        // reading discretes
        ctx.clear_coils();
        ctx.clear_discretes();
        for c in 200..200 + coils.len() {
            ctx.set_discrete(c as u16, coils[c - 200]).unwrap();
        }
        let mut mreq = ModbusRequest::new(3, *proto);
        let mut request = Vec::new();
        mreq.generate_get_discretes(200, coils.len() as u16, &mut request)
            .unwrap();
        let mut response = Vec::new();
        let mut framebuf: ModbusFrameBuf = [0; 256];
        if *proto == ModbusProto::Rtu {
            let mut ascii_frame = Vec::new();
            generate_ascii_frame(&request, &mut ascii_frame).unwrap();
            for i in 0..framebuf.len() {
                framebuf[i] = 0
            }
            parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
        } else {
            for i in 0..request.len() {
                framebuf[i] = request[i];
            }
        }
        let mut frame = ModbusFrame::new(3, &framebuf, *proto, &mut response);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, true);
        frame.process_read(&mut *ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        let mut result = Vec::new();
        mreq.parse_bool(&response, &mut result).unwrap();
        assert_eq!(result, coils);

        // set single coil
        ctx.clear_coils();
        ctx.clear_discretes();
        let mut mreq = ModbusRequest::new(4, *proto);
        let mut request = Vec::new();
        mreq.generate_set_coil(100, true, &mut request).unwrap();
        let mut response = Vec::new();
        let mut framebuf: ModbusFrameBuf = [0; 256];
        if *proto == ModbusProto::Rtu {
            let mut ascii_frame = Vec::new();
            generate_ascii_frame(&request, &mut ascii_frame).unwrap();
            for i in 0..framebuf.len() {
                framebuf[i] = 0
            }
            parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
        } else {
            for i in 0..request.len() {
                framebuf[i] = request[i];
            }
        }
        let mut frame = ModbusFrame::new(4, &framebuf, *proto, &mut response);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut *ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        mreq.parse_ok(&response).unwrap();
        assert_eq!(ctx.get_coil(100).unwrap(), true);

        // set coils oob
        let mut mreq = ModbusRequest::new(4, *proto);
        let mut request = Vec::new();
        mreq.generate_set_coil(10001, true, &mut request).unwrap();
        let mut response = Vec::new();
        let mut framebuf: ModbusFrameBuf = [0; 256];
        if *proto == ModbusProto::Rtu {
            let mut ascii_frame = Vec::new();
            generate_ascii_frame(&request, &mut ascii_frame).unwrap();
            for i in 0..framebuf.len() {
                framebuf[i] = 0
            }
            parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
        } else {
            for i in 0..request.len() {
                framebuf[i] = request[i];
            }
        }
        let mut frame = ModbusFrame::new(4, &framebuf, *proto, &mut response);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut *ctx).unwrap();
        assert_eq!(frame.error, 2);
        frame.finalize_response().unwrap();
        assert_eq!(
            mreq.parse_ok(&response).err().unwrap(),
            ErrorKind::IllegalDataAddress
        );

        // set holdings bulk
        ctx.clear_holdings();
        let mut mreq = ModbusRequest::new(2, *proto);
        let mut request = Vec::new();
        mreq.generate_set_holdings_bulk(100, &holdings, &mut request)
            .unwrap();
        let mut response = Vec::new();
        let mut framebuf: ModbusFrameBuf = [0; 256];
        if *proto == ModbusProto::Rtu {
            let mut ascii_frame = Vec::new();
            generate_ascii_frame(&request, &mut ascii_frame).unwrap();
            for i in 0..framebuf.len() {
                framebuf[i] = 0
            }
            parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
        } else {
            for i in 0..request.len() {
                framebuf[i] = request[i];
            }
        }
        let mut frame = ModbusFrame::new(2, &framebuf, *proto, &mut response);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut *ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        mreq.parse_ok(&response).unwrap();
        for i in 100..100 + holdings.len() {
            assert_eq!(ctx.get_holding(i as u16).unwrap(), holdings[i - 100]);
        }

        // reading holdings
        let mut mreq = ModbusRequest::new(3, *proto);
        let mut request = Vec::new();
        mreq.generate_get_holdings(100, holdings.len() as u16, &mut request)
            .unwrap();
        let mut response = Vec::new();
        let mut framebuf: ModbusFrameBuf = [0; 256];
        if *proto == ModbusProto::Rtu {
            let mut ascii_frame = Vec::new();
            generate_ascii_frame(&request, &mut ascii_frame).unwrap();
            for i in 0..framebuf.len() {
                framebuf[i] = 0
            }
            parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
        } else {
            for i in 0..request.len() {
                framebuf[i] = request[i];
            }
        }
        let mut frame = ModbusFrame::new(3, &framebuf, *proto, &mut response);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, true);
        frame.process_read(&mut *ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        let mut result = Vec::new();
        mreq.parse_u16(&response, &mut result).unwrap();
        assert_eq!(result, holdings);

        // set holdings string
        ctx.clear_holdings();
        let mut mreq = ModbusRequest::new(2, *proto);
        let mut request = Vec::new();
        mreq.generate_set_holdings_string(100, &holdstr, &mut request)
            .unwrap();
        let mut response = Vec::new();
        let mut framebuf: ModbusFrameBuf = [0; 256];
        if *proto == ModbusProto::Rtu {
            let mut ascii_frame = Vec::new();
            generate_ascii_frame(&request, &mut ascii_frame).unwrap();
            for i in 0..framebuf.len() {
                framebuf[i] = 0
            }
            parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
        } else {
            for i in 0..request.len() {
                framebuf[i] = request[i];
            }
        }
        let mut frame = ModbusFrame::new(2, &framebuf, *proto, &mut response);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut *ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        mreq.parse_ok(&response).unwrap();

        // reading holdings string
        let mut mreq = ModbusRequest::new(3, *proto);
        let mut request = Vec::new();
        mreq.generate_get_holdings(100, holdstr.len() as u16, &mut request)
            .unwrap();
        let mut response = Vec::new();
        let mut framebuf: ModbusFrameBuf = [0; 256];
        if *proto == ModbusProto::Rtu {
            let mut ascii_frame = Vec::new();
            generate_ascii_frame(&request, &mut ascii_frame).unwrap();
            for i in 0..framebuf.len() {
                framebuf[i] = 0
            }
            parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
        } else {
            for i in 0..request.len() {
                framebuf[i] = request[i];
            }
        }
        let mut frame = ModbusFrame::new(3, &framebuf, *proto, &mut response);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, true);
        frame.process_read(&mut *ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        let mut result = String::new();
        mreq.parse_string(&response, &mut result).unwrap();
        assert_eq!(result, holdstr);

        // reading inputs
        ctx.clear_holdings();
        ctx.clear_inputs();
        for c in 200..200 + holdings.len() {
            ctx.set_input(c as u16, holdings[c - 200]).unwrap();
        }
        let mut mreq = ModbusRequest::new(3, *proto);
        let mut request = Vec::new();
        mreq.generate_get_inputs(200, holdings.len() as u16, &mut request)
            .unwrap();
        let mut response = Vec::new();
        let mut framebuf: ModbusFrameBuf = [0; 256];
        if *proto == ModbusProto::Rtu {
            let mut ascii_frame = Vec::new();
            generate_ascii_frame(&request, &mut ascii_frame).unwrap();
            for i in 0..framebuf.len() {
                framebuf[i] = 0
            }
            parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
        } else {
            for i in 0..request.len() {
                framebuf[i] = request[i];
            }
        }
        let mut frame = ModbusFrame::new(3, &framebuf, *proto, &mut response);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, true);
        frame.process_read(&mut *ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        let mut result = Vec::new();
        mreq.parse_u16(&response, &mut result).unwrap();
        assert_eq!(result, holdings);

        // set single holding
        ctx.clear_holdings();
        ctx.clear_inputs();
        let mut mreq = ModbusRequest::new(4, *proto);
        let mut request = Vec::new();
        mreq.generate_set_holding(100, 7777, &mut request).unwrap();
        let mut response = Vec::new();
        let mut framebuf: ModbusFrameBuf = [0; 256];
        if *proto == ModbusProto::Rtu {
            let mut ascii_frame = Vec::new();
            generate_ascii_frame(&request, &mut ascii_frame).unwrap();
            for i in 0..framebuf.len() {
                framebuf[i] = 0
            }
            parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
        } else {
            for i in 0..request.len() {
                framebuf[i] = request[i];
            }
        }
        let mut frame = ModbusFrame::new(4, &framebuf, *proto, &mut response);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut *ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        mreq.parse_ok(&response).unwrap();
        assert_eq!(ctx.get_coil(100).unwrap(), true);

        // set holding oob
        let mut mreq = ModbusRequest::new(4, *proto);
        let mut request = Vec::new();
        mreq.generate_set_holding(10001, 0x7777, &mut request)
            .unwrap();
        let mut response = Vec::new();
        let mut framebuf: ModbusFrameBuf = [0; 256];
        if *proto == ModbusProto::Rtu {
            let mut ascii_frame = Vec::new();
            generate_ascii_frame(&request, &mut ascii_frame).unwrap();
            for i in 0..framebuf.len() {
                framebuf[i] = 0
            }
            parse_ascii_frame(&ascii_frame, ascii_frame.len(), &mut framebuf, 0).unwrap();
        } else {
            for i in 0..request.len() {
                framebuf[i] = request[i];
            }
        }
        let mut frame = ModbusFrame::new(4, &framebuf, *proto, &mut response);
        frame.parse().unwrap();
        assert_eq!(frame.response_required, true);
        assert_eq!(frame.processing_required, true);
        assert_eq!(frame.error, 0);
        assert_eq!(frame.readonly, false);
        frame.process_write(&mut *ctx).unwrap();
        assert_eq!(frame.error, 2);
        frame.finalize_response().unwrap();
        assert_eq!(
            mreq.parse_ok(&response).err().unwrap(),
            ErrorKind::IllegalDataAddress
        );
    }
}
