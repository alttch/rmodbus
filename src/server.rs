#[path = "context.rs"]
pub mod context;

use super::{VectorTrait, ErrorKind};

/// Standard Modbus frame
///
/// As max length of Modbus frame + headers is always 256 bytes or less, the frame is a fixed [u8;
/// 256] array.
pub type ModbusFrame = [u8; 256];

/// Modbus protocol selection for frame processing
///
/// * for **TcpUdp**, Modbus TCP headers are parsed / added to replies
/// * for **Rtu**, frame checksums are verified / added to repies
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum ModbusProto {
    Rtu,
    TcpUdp,
}

fn calc_rtu_crc(frame: &[u8], data_length: u8) -> u16 {
    let mut crc: u16 = 0xffff;
    for pos in 0..data_length as usize {
        crc = crc ^ frame[pos] as u16;
        for _ in (0..8).rev() {
            if (crc & 0x0001) != 0 {
                crc = crc >> 1;
                crc = crc ^ 0xA001;
            } else {
                crc = crc >> 1;
            }
        }
    }
    return crc;
}

/// Process Modbus frame
///
/// Simple example of Modbus/UDP blocking server:
///
/// ```
///use std::net::UdpSocket;
///
///use rmodbus::server::{ModbusFrame, ModbusProto, process_frame};
///
///pub fn udpserver(unit: u8, listen: &str) {
///    let socket = UdpSocket::bind(listen).unwrap();
///    loop {
///        // init frame buffer
///        let mut buf: ModbusFrame = [0; 256];
///        let (_amt, src) = socket.recv_from(&mut buf).unwrap();
///        // Send frame for processing - modify context for write frames and get response
///        let mut response: Vec<u8> = Vec::new(); // use FixedVec for nostd
///        if process_frame(unit, &buf, ModbusProto::TcpUdp, &mut response).is_err() {
///            // continue loop (or exit function) if there's nothing to send as the reply
///            continue;
///            }
///        if !response.is_empty() {
///            socket.send_to(response.as_slice(), &src).unwrap();
///            }
///        }
///    }
/// ```
///
/// There are also [examples of TCP and
/// RTU](https://github.com/alttch/rmodbus/tree/master/examples/example-server/src)
///
/// For broadcast requests, the response vector is empty
///
/// The function returns Error in cases:
///
/// * **rmodbus::ErrorKind::FrameBroken**: the frame header is absolutely incorrect and there's no
///   way to form a valid Modbus error reply
///
/// * **rmodbus::ErrorKind::FrameCRCError**: frame CRC error (Modbus RTU)
///
/// * **rmodbus::ErrorKind::OOB**: for nostd only, unable to write response into FixedVec
pub fn process_frame<V: VectorTrait<u8>>(
    unit_id: u8,
    frame: &ModbusFrame,
    proto: ModbusProto,
    response: &mut V,
) -> Result<(), ErrorKind> {
    let start_frame: usize;
    if proto == ModbusProto::TcpUdp {
        //let tr_id = u16::from_be_bytes([frame[0], frame[1]]);
        let proto_id = u16::from_be_bytes([frame[2], frame[3]]);
        let length = u16::from_be_bytes([frame[4], frame[5]]);
        if proto_id != 0 || length < 6 {
            return Err(ErrorKind::FrameBroken);
        }
        start_frame = 6;
    } else {
        start_frame = 0;
    }
    response.clear_all();
    let unit = frame[start_frame];
    let broadcast = unit == 0 || unit == 255; // some clients send broadcast to 0xff
    if !broadcast && unit != unit_id {
        return Ok(());
    }
    if !broadcast && proto == ModbusProto::TcpUdp {
        // copy 4 bytes: tr id and proto
        if response.add_bulk(&frame[0..4]).is_err() {
            return Err(ErrorKind::OOB);
        }
    }
    let func = frame[start_frame + 1];
    macro_rules! check_frame_crc {
        ($len:expr) => {
            proto == ModbusProto::TcpUdp
                || calc_rtu_crc(frame, $len)
                    == u16::from_le_bytes([frame[$len as usize], frame[$len as usize + 1]]);
        };
    }
    macro_rules! response_error {
        ($err:expr) => {
            match proto {
                ModbusProto::TcpUdp => {
                    if response
                        .add_bulk(&[0, 3, frame[7], frame[8] + 0x80, $err])
                        .is_err()
                    {
                        return Err(ErrorKind::OOB);
                    }
                }
                ModbusProto::Rtu => {
                    if response
                        .add_bulk(&[frame[0], frame[1] + 0x80, $err])
                        .is_err()
                    {
                        return Err(ErrorKind::OOB);
                    }
                }
            }
        };
    }
    macro_rules! response_set_data_len {
        ($len:expr) => {
            if proto == ModbusProto::TcpUdp {
                if response.add_bulk(&($len as u16).to_be_bytes()).is_err() {
                    return Err(ErrorKind::OOB);
                }
            }
        };
    }
    macro_rules! finalize_response {
        () => {
            if proto == ModbusProto::Rtu {
                let crc = calc_rtu_crc(&response.get_slice(), response.get_len() as u8);
                if response.add_bulk(&crc.to_le_bytes()).is_err() {
                    return Err(ErrorKind::OOB);
                }
            }
        };
    }
    if func == 1 || func == 2 {
        // funcs 1 - 2
        // read coils / discretes
        if broadcast {
            return Ok(());
        }
        if !check_frame_crc!(6) {
            return Err(ErrorKind::FrameCRCError);
        }
        let count = u16::from_be_bytes([frame[start_frame + 4], frame[start_frame + 5]]);
        if count > 2000 {
            response_error!(0x03);
            finalize_response!();
            return Ok(());
        }
        let reg = u16::from_be_bytes([frame[start_frame + 2], frame[start_frame + 3]]);
        let mut data_len = count >> 3;
        if count % 8 != 0 {
            data_len = data_len + 1;
        }
        response_set_data_len!(data_len + 3);
        if response
            .add_bulk(&frame[start_frame..start_frame + 2]) // 2b unit and func
            .is_err()
        {
            return Err(ErrorKind::OOB);
        }
        if response.add(data_len as u8).is_err() {
            // 1b data len
            return Err(ErrorKind::OOB);
        }
        let ctx = lock_mutex!(context::CONTEXT);
        let result = match func {
            1 => context::get_bools_as_u8(reg, count, &ctx.coils, response),
            2 => context::get_bools_as_u8(reg, count, &ctx.discretes, response),
            _ => panic!(), // never reaches
        };
        drop(ctx);
        return match result {
            Ok(_) => {
                finalize_response!();
                Ok(())
            }
            Err(_) => {
                response.cut_end(5, 0);
                response_error!(0x02);
                finalize_response!();
                Ok(())
            }
        };
    } else if func == 3 || func == 4 {
        // funcs 3 - 4
        // read holdings / inputs
        if broadcast {
            return Ok(());
        }
        if !check_frame_crc!(6) {
            return Err(ErrorKind::FrameCRCError);
        }
        let count = u16::from_be_bytes([frame[start_frame + 4], frame[start_frame + 5]]);
        if count > 125 {
            response_error!(0x03);
            finalize_response!();
            return Ok(());
        }
        let reg = u16::from_be_bytes([frame[start_frame + 2], frame[start_frame + 3]]);
        let data_len = count << 1;
        response_set_data_len!(data_len + 3);
        if response
            .add_bulk(&frame[start_frame..start_frame + 2]) // 2b unit and func
            .is_err()
        {
            return Err(ErrorKind::OOB);
        }
        if response.add(data_len as u8).is_err() {
            // 1b data len
            return Err(ErrorKind::OOB);
        }
        let ctx = lock_mutex!(context::CONTEXT);
        let result = match func {
            3 => context::get_regs_as_u8(reg, count, &ctx.holdings, response),
            4 => context::get_regs_as_u8(reg, count, &ctx.inputs, response),
            _ => panic!(), // never reaches
        };
        drop(ctx);
        return match result {
            Ok(_) => {
                finalize_response!();
                Ok(())
            }
            Err(_) => {
                response.cut_end(5, 0);
                response_error!(0x02);
                finalize_response!();
                Ok(())
            }
        };
    } else if func == 5 {
        // func 5
        // write single coil
        if !check_frame_crc!(6) {
            return Err(ErrorKind::FrameCRCError);
        }
        let reg = u16::from_be_bytes([frame[start_frame + 2], frame[start_frame + 3]]);
        let val: bool;
        match u16::from_be_bytes([frame[start_frame + 4], frame[start_frame + 5]]) {
            0xff00 => val = true,
            0x0000 => val = false,
            _ => {
                if broadcast {
                    return Ok(());
                } else {
                    response_error!(0x03);
                    finalize_response!();
                    return Ok(());
                }
            }
        };
        let result = context::set(reg, val, &mut lock_mutex!(context::CONTEXT).coils);
        if broadcast {
            return Ok(());
        } else if result.is_err() {
            response_error!(0x02);
            finalize_response!();
            return Ok(());
        } else {
            response_set_data_len!(6);
            // 6b unit, func, reg, val
            if response
                .add_bulk(&frame[start_frame..start_frame + 6])
                .is_err()
            {
                return Err(ErrorKind::OOB);
            }
            finalize_response!();
            return Ok(());
        }
    } else if func == 6 {
        // func 6
        // write single register
        if !check_frame_crc!(6) {
            return Err(ErrorKind::FrameCRCError);
        }
        let reg = u16::from_be_bytes([frame[start_frame + 2], frame[start_frame + 3]]);
        let val = u16::from_be_bytes([frame[start_frame + 4], frame[start_frame + 5]]);
        let result = context::set(reg, val, &mut lock_mutex!(context::CONTEXT).holdings);
        if broadcast {
            return Ok(());
        } else if result.is_err() {
            response_error!(0x02);
            finalize_response!();
            return Ok(());
        } else {
            response_set_data_len!(6);
            // 6b unit, func, reg, val
            if response
                .add_bulk(&frame[start_frame..start_frame + 6])
                .is_err()
            {
                return Err(ErrorKind::OOB);
            }
            finalize_response!();
            return Ok(());
        }
    } else if func == 0x0f || func == 0x10 {
        // funcs 15 & 16
        // write multiple coils / registers
        let bytes = frame[start_frame + 6];
        if !check_frame_crc!(7 + bytes) {
            return Err(ErrorKind::FrameCRCError);
        }
        if bytes > 242 {
            if broadcast {
                return Ok(());
            } else {
                response_error!(0x03);
                finalize_response!();
                return Ok(());
            }
        }
        let reg = u16::from_be_bytes([frame[start_frame + 2], frame[start_frame + 3]]);
        let count = u16::from_be_bytes([frame[start_frame + 4], frame[start_frame + 5]]);
        let result = match func {
            0x0f => context::set_bools_from_u8(
                reg,
                count,
                &frame[start_frame + 7..start_frame + 7 + bytes as usize],
                &mut lock_mutex!(context::CONTEXT).coils,
            ),
            0x10 => context::set_regs_from_u8(
                reg,
                &frame[start_frame + 7..start_frame + 7 + bytes as usize],
                &mut lock_mutex!(context::CONTEXT).holdings,
            ),
            _ => panic!(), // never reaches
        };
        if broadcast {
            return Ok(());
        } else {
            match result {
                Ok(_) => {
                    response_set_data_len!(6);
                    // 6b unit, f, reg, cnt
                    if response
                        .add_bulk(&frame[start_frame..start_frame + 6])
                        .is_err()
                    {
                        return Err(ErrorKind::OOB);
                    }
                    finalize_response!();
                    return Ok(());
                }
                Err(_) => {
                    response_error!(0x02);
                    finalize_response!();
                    return Ok(());
                }
            }
        }
    } else {
        // function unsupported
        response_error!(0x01);
        finalize_response!();
        return Ok(());
    }
}
