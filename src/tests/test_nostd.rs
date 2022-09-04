use fixedvec::alloc_stack;

use crate::client::*;
use crate::server::context::*;
use crate::server::*;
use crate::*;

use fixedvec::FixedVec;
use rand::Rng;
use spin::RwLock;

lazy_static! {
    pub static ref CTX: RwLock<ModbusContext> = RwLock::new(ModbusContext::new());
}

#[test]
fn test_nostd_coil_get_set_bulk() {
    let mut ctx = CTX.write();
    let mut data_mem = alloc_stack!([bool; CONTEXT_SIZE]);
    let mut data = FixedVec::new(&mut data_mem);
    let mut result_mem = alloc_stack!([bool; CONTEXT_SIZE]);
    let mut result = FixedVec::new(&mut result_mem);
    data.push_all(&[true; 2]).unwrap();
    ctx.set_coils_bulk(5, &data.as_slice()).unwrap();
    ctx.get_coils_bulk(5, 2, &mut result).unwrap();
    assert_eq!(result, data);

    data.clear();
    result.clear();

    data.push_all(&[true; 18]).unwrap();
    ctx.set_coils_bulk(25, &data.as_slice()).unwrap();
    ctx.get_coils_bulk(25, 18, &mut result).unwrap();
    assert_eq!(result, data);
}

#[test]
fn test_nostd_get_holding_set_bulk() {
    let mut ctx = CTX.write();
    let mut data_mem = alloc_stack!([u16; CONTEXT_SIZE]);
    let mut data = FixedVec::new(&mut data_mem);
    let mut result_mem = alloc_stack!([u16; CONTEXT_SIZE]);
    let mut result = FixedVec::new(&mut result_mem);

    ctx.clear_holdings();

    data.push_all(&[0x77; 2]).unwrap();
    ctx.set_holdings_bulk(5, &data.as_slice()).unwrap();
    ctx.get_holdings_bulk(5, 2, &mut result).unwrap();
    assert_eq!(result, data);

    data.clear();
    result.clear();

    data.push_all(&[0x33; 18]).unwrap();
    ctx.set_holdings_bulk(25, &data.as_slice()).unwrap();
    ctx.get_holdings_bulk(25, 18, &mut result).unwrap();
    assert_eq!(result, data);
}

#[test]
fn test_nostd_get_bools_as_u8() {
    let mut ctx = CTX.write();
    let mut data_mem = alloc_stack!([bool; CONTEXT_SIZE]);
    let mut data = FixedVec::new(&mut data_mem);
    ctx.clear_coils();
    data.push_all(&[true, true, true, true, true, true, false, false])
        .unwrap();
    ctx.set_coils_bulk(0, &data.as_slice()).unwrap();
    let mut result_mem = alloc_stack!([u8; CONTEXT_SIZE / 8]);
    let mut result = FixedVec::new(&mut result_mem);
    ctx.get_coils_as_u8(0, 6, &mut result).unwrap();
    assert_eq!(*result.get(0).unwrap(), 0b00111111);
    result.clear();
    ctx.get_coils_as_u8(0, 5, &mut result).unwrap();
    assert_eq!(*result.get(0).unwrap(), 0b00011111);
    result.clear();
    data.clear();
    data.push_all(&[true, true, false, true, true, true, true, true])
        .unwrap();
    ctx.set_coils_bulk(0, &data.as_slice()).unwrap();
    let mut result_mem = alloc_stack!([u8; CONTEXT_SIZE / 8]);
    let mut result = FixedVec::new(&mut result_mem);
    ctx.get_coils_as_u8(0, 6, &mut result).unwrap();
    assert_eq!(*result.get(0).unwrap(), 0b00111011);
    result.clear();
    ctx.get_coils_as_u8(0, 5, &mut result).unwrap();
    assert_eq!(*result.get(0).unwrap(), 0b00011011);
    result.clear();
    data.clear();
    data.push_all(&[
        true, true, false, true, true, true, true, true, // byte 1
        true, true, true, true, false, false, true, false, // byte 2
        false, false, false, true, false, true, // byte 3
    ])
    .unwrap();
    ctx.set_coils_bulk(0, &data.as_slice()).unwrap();
    let mut result_mem = alloc_stack!([u8; CONTEXT_SIZE / 8]);
    let mut result = FixedVec::new(&mut result_mem);
    ctx.get_coils_as_u8(0, 22, &mut result).unwrap();
    assert_eq!(*result.get(0).unwrap(), 0b11111011);
    assert_eq!(*result.get(1).unwrap(), 0b01001111);
    assert_eq!(*result.get(2).unwrap(), 0b101000);
}

#[test]
fn test_nostd_get_set_regs_as_u8() {
    let mut ctx = CTX.write();
    ctx.clear_holdings();
    let mut data_mem = alloc_stack!([u16; CONTEXT_SIZE]);
    let mut data = FixedVec::new(&mut data_mem);
    data.push_all(&[2, 45, 4559, 31, 394, 1, 9, 7, 0, 1, 9])
        .unwrap();
    ctx.set_holdings_bulk(0, &data.as_slice()).unwrap();
    let mut result_mem = alloc_stack!([u8; CONTEXT_SIZE]);
    let mut result = FixedVec::new(&mut result_mem);
    ctx.get_holdings_as_u8(0, data.len() as u16, &mut result)
        .unwrap();
    assert_eq!(result[0], 0);
    assert_eq!(result[1], 2);
    for i in 0..10 {
        ctx.set_holding(i, 0).unwrap();
    }
    ctx.set_holdings_from_u8(0, &result.as_slice()).unwrap();
    let mut result_mem = alloc_stack!([u16; CONTEXT_SIZE]);
    let mut result = FixedVec::new(&mut result_mem);
    ctx.get_holdings_bulk(0, data.len() as u16, &mut result)
        .unwrap();
    assert_eq!(result, data);
}

#[test]
fn test_nostd_get_set_bools_as_u8() {
    let mut ctx = CTX.write();
    ctx.clear_coils();
    let mut data_mem = alloc_stack!([bool; CONTEXT_SIZE]);
    let mut data = FixedVec::new(&mut data_mem);
    data.push_all(&[
        true, true, true, false, true, true, true, true, true, false, false, false, false, false,
    ])
    .unwrap();
    ctx.set_coils_bulk(0, &data.as_slice()).unwrap();
    ctx.set_coil(data.len() as u16, true).unwrap();
    ctx.set_coil(data.len() as u16 + 1, false).unwrap();
    ctx.set_coil(data.len() as u16 + 2, true).unwrap();
    let mut result_mem = alloc_stack!([u8; CONTEXT_SIZE]);
    let mut result = FixedVec::new(&mut result_mem);
    ctx.get_coils_as_u8(0, data.len() as u16, &mut result)
        .unwrap();
    ctx.set_coils_from_u8(0, data.len() as u16, &result.as_slice())
        .unwrap();
    let mut result_mem = alloc_stack!([bool; CONTEXT_SIZE]);
    let mut result = FixedVec::new(&mut result_mem);
    ctx.get_coils_bulk(0, data.len() as u16, &mut result)
        .unwrap();
    assert_eq!(result, data);
    result.clear();
    data.push(true).unwrap();
    data.push(false).unwrap();
    data.push(true).unwrap();
    ctx.get_coils_bulk(0, data.len() as u16, &mut result)
        .unwrap();
    assert_eq!(result, data);
}

#[test]
fn test_nostd_dump_restore() {
    let mut ctx = CTX.write();
    let mut rng = rand::thread_rng();
    let mut coils_mem = alloc_stack!([bool; CONTEXT_SIZE]);
    let mut discretes_mem = alloc_stack!([bool; CONTEXT_SIZE]);
    let mut inputs_mem = alloc_stack!([u16; CONTEXT_SIZE]);
    let mut holdings_mem = alloc_stack!([u16; CONTEXT_SIZE]);
    let mut mycoils = FixedVec::new(&mut coils_mem);
    let mut mydiscretes = FixedVec::new(&mut discretes_mem);
    let mut myinputs = FixedVec::new(&mut inputs_mem);
    let mut myholdings = FixedVec::new(&mut holdings_mem);
    for _ in 0..CONTEXT_SIZE {
        mycoils.push(rng.gen()).unwrap();
        mydiscretes.push(rng.gen()).unwrap();
        myholdings.push(rng.gen()).unwrap();
        myinputs.push(rng.gen()).unwrap();
    }
    ctx.clear_all();
    ctx.set_coils_bulk(0, &mycoils.as_slice()).unwrap();
    ctx.set_discretes_bulk(0, &mydiscretes.as_slice()).unwrap();
    ctx.set_holdings_bulk(0, &myholdings.as_slice()).unwrap();
    ctx.set_inputs_bulk(0, &myinputs.as_slice()).unwrap();
    let mut dump_mem = alloc_stack!([u8; CONTEXT_SIZE * 17 / 4]);
    let mut dump = FixedVec::new(&mut dump_mem);
    for i in 0..CONTEXT_SIZE * 17 / 4 {
        dump.push(ctx.get_cell(i as u16).unwrap()).unwrap();
    }
    ctx.clear_all();
    let mut offset = 0;
    for value in &dump {
        ctx.set_cell(offset, *value).unwrap();
        offset = offset + 1;
    }
    let mut result_mem = alloc_stack!([bool; CONTEXT_SIZE]);
    let mut result = FixedVec::new(&mut result_mem);
    ctx.get_coils_bulk(0, CONTEXT_SIZE as u16, &mut result)
        .unwrap();
    assert_eq!(result, mycoils);
    result.clear();
    ctx.get_discretes_bulk(0, CONTEXT_SIZE as u16, &mut result)
        .unwrap();
    assert_eq!(result, mydiscretes);
    result.clear();

    let mut result_mem = alloc_stack!([u16; CONTEXT_SIZE]);
    let mut result = FixedVec::new(&mut result_mem);
    ctx.get_inputs_bulk(0, CONTEXT_SIZE as u16, &mut result)
        .unwrap();
    assert_eq!(result, myinputs);
    result.clear();
    ctx.get_holdings_bulk(0, CONTEXT_SIZE as u16, &mut result)
        .unwrap();
    assert_eq!(result, myholdings);
    result.clear();

    let mut dump2_mem = alloc_stack!([u8; CONTEXT_SIZE * 17 / 4]);
    let mut dump2 = FixedVec::new(&mut dump2_mem);
    for value in ctx.iter() {
        dump2.push(value).unwrap();
    }
    assert_eq!(dump, dump2);
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
    return frame;
}

#[test]
fn test_nostd_frame() {
    let mut ctx = CTX.write();
    ctx.clear_coils();
    ctx.clear_all();
    let mut result_mem = alloc_stack!([u8; 256]);
    let mut result = FixedVec::new(&mut result_mem);
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
    assert_eq!(frame.error, 0);
    assert_eq!(frame.readonly, true);
    frame.process_read(&ctx).unwrap();
    assert_eq!(frame.error, 0);
    frame.finalize_response().unwrap();
    assert_eq!(result.as_slice(), response);
    //check result OOB
    let mut result_mem = alloc_stack!([u8; 10]);
    for i in 0..10 {
        let mut result = FixedVec::new(&mut result_mem[..i]);
        let mut frame = ModbusFrame::new(1, &framebuf, ModbusProto::TcpUdp, &mut result);
        match frame.parse() {
            Ok(_) => {
                if i > 3 {
                    match frame.process_read(&ctx) {
                        Ok(_) => panic!("{:x?}", result),
                        Err(e) => match e {
                            ErrorKind::OOB => {}
                            _ => panic!("{:?}", e),
                        },
                    }
                } else {
                    panic!("{:x?}", result)
                }
            }
            Err(e) => match e {
                ErrorKind::OOB => {}
                _ => panic!("{:?}", e),
            },
        }
    }
}

#[test]
fn test_nostd_client() {
    let mut ctx = CTX.write();
    ctx.clear_discretes();
    let coils = [
        true, true, true, false, true, true, false, true, true, false, true,
    ];
    let protos = [ModbusProto::TcpUdp, ModbusProto::Rtu, ModbusProto::Ascii];

    for proto in &protos {
        // set coils bulk
        ctx.clear_coils();
        let mut mreq = ModbusRequest::new(2, *proto);
        let mut request_mem = alloc_stack!([u8; 256]);
        let mut request = FixedVec::new(&mut request_mem);
        mreq.generate_set_coils_bulk(100, &coils, &mut request)
            .unwrap();
        let mut response_mem = alloc_stack!([u8; 256]);
        let mut response = FixedVec::new(&mut response_mem);
        let mut framebuf: ModbusFrameBuf = [0; 256];
        if *proto == ModbusProto::Rtu {
            let mut ascii_mem = alloc_stack!([u8; 1024]);
            let mut ascii_frame = FixedVec::new(&mut ascii_mem);
            generate_ascii_frame(&request.as_slice(), &mut ascii_frame).unwrap();
            for i in 0..framebuf.len() {
                framebuf[i] = 0
            }
            parse_ascii_frame(&ascii_frame.as_slice(), ascii_frame.len(), &mut framebuf, 0)
                .unwrap();
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
        frame.process_write(&mut ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        mreq.parse_ok(&response.as_slice()).unwrap();
        for i in 100..100 + coils.len() {
            assert_eq!(ctx.get_coil(i as u16).unwrap(), coils[i - 100]);
        }

        // reading coils
        let mut mreq = ModbusRequest::new(3, *proto);
        let mut request_mem = alloc_stack!([u8; 256]);
        let mut request = FixedVec::new(&mut request_mem);
        mreq.generate_get_coils(100, coils.len() as u16, &mut request)
            .unwrap();
        let mut response_mem = alloc_stack!([u8; 256]);
        let mut response = FixedVec::new(&mut response_mem);
        let mut framebuf: ModbusFrameBuf = [0; 256];
        if *proto == ModbusProto::Rtu {
            let mut ascii_mem = alloc_stack!([u8; 1024]);
            let mut ascii_frame = FixedVec::new(&mut ascii_mem);
            generate_ascii_frame(&request.as_slice(), &mut ascii_frame).unwrap();
            for i in 0..framebuf.len() {
                framebuf[i] = 0
            }
            parse_ascii_frame(&ascii_frame.as_slice(), ascii_frame.len(), &mut framebuf, 0)
                .unwrap();
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
        frame.process_read(&mut ctx).unwrap();
        assert_eq!(frame.error, 0);
        frame.finalize_response().unwrap();
        let mut result_mem = alloc_stack!([bool; 256]);
        let mut result = FixedVec::new(&mut result_mem);
        mreq.parse_bool(&response.as_slice(), &mut result).unwrap();
        assert_eq!(result.as_slice(), coils);
    }
}
