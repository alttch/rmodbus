#[path = "context.rs"]
pub mod context;

use super::{ErrorKind, VectorTrait};

/// Standard Modbus frame
///
/// As max length of Modbus frame + headers is always 256 bytes or less, the frame is a fixed [u8;
/// 256] array.
pub type ModbusFrame = [u8; 256];

/// Modbus protocol selection for frame processing
///
/// * for **TcpUdp**, Modbus TCP headers are parsed / added to replies
/// * for **Rtu**, frame checksums are verified / added to replies
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum ModbusProto {
    Rtu,
    Ascii,
    TcpUdp,
}

fn calc_crc16(frame: &[u8], data_length: u8) -> u16 {
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

fn calc_lrc(frame: &[u8], data_length: u8) -> u8 {
    let mut lrc: i32 = 0;
    for i in 0..data_length {
        lrc = lrc - frame[i as usize] as i32;
    }
    return lrc as u8;
}

fn chr_to_hex(c: u8) -> Result<u8, ErrorKind> {
    if c >= 48 && c <= 57 {
        return Ok(c - 48);
    } else if c >= 65 && c <= 70 {
        return Ok(c - 55);
    } else {
        return Err(ErrorKind::FrameBroken);
    }
}

fn hex_to_chr(h: u8) -> u8 {
    if h < 10 {
        return h + 48;
    } else {
        return h + 55;
    }
}

/// Parse ASCII Modbus frame
///
/// data - input buffer
/// data_len - how many bytes to parse in buffer
/// frame - frame buffer
/// frame_pos - position in frame buffer to write
///
/// The frame can be parsed fully or partially (use frame_pos)
///
/// Errors:
///
/// * **OOB** input is larger than frame buffer (starting from frame_pos)
/// * **FrameBroken** unable to decode input hex string
pub fn parse_ascii_frame(
    data: &[u8],
    data_len: usize,
    frame: &mut ModbusFrame,
    frame_pos: u8,
) -> Result<u8, ErrorKind> {
    let mut dpos = match data[0] {
        58 => 1, // ':'
        _ => 0,
    };
    let mut cpos = frame_pos;
    while dpos < data_len {
        if cpos == 255 {
            return Err(ErrorKind::OOB);
        }
        let ch = data[dpos];
        if ch == 10 || ch == 13 || ch == 0 {
            break;
        }
        let c = match chr_to_hex(data[dpos]) {
            Ok(v) => v,
            Err(_) => return Err(ErrorKind::FrameBroken),
        };
        dpos = dpos + 1;
        if dpos >= data_len {
            return Err(ErrorKind::OOB);
        }
        let c2 = match chr_to_hex(data[dpos]) {
            Ok(v) => v,
            Err(_) => return Err(ErrorKind::FrameBroken),
        };
        frame[cpos as usize] = c * 0x10 + c2;
        dpos = dpos + 1;
        cpos = cpos + 1;
    }
    return Ok(cpos - frame_pos - 1);
}

/// Generate ASCII frame
///
/// Generates ASCII frame from binary response, made by "process_frame" function (response must be
/// supplited as slice)
pub fn generate_ascii_frame<V: VectorTrait<u8>>(
    data: &[u8],
    result: &mut V,
) -> Result<(), ErrorKind> {
    result.clear_all();
    if result.add(58).is_err() {
        return Err(ErrorKind::OOB);
    }
    for d in data {
        if result.add(hex_to_chr(d >> 4)).is_err() {
            return Err(ErrorKind::OOB);
        }
        if result.add(hex_to_chr(*d & 0xf)).is_err() {
            return Err(ErrorKind::OOB);
        }
    }
    if result.add(0x0D).is_err() {
        return Err(ErrorKind::OOB);
    }
    if result.add(0x0A).is_err() {
        return Err(ErrorKind::OOB);
    }
    return Ok(());
}

/// Guess serial frame length
///
/// Serial frames are often read either byte-by-byte or by DMA. In the both cases, the exact frame
/// length should be known.
///
/// How to use: read at least first 7 bytes into buffer and call the function to guess the total
/// frame length. The remaining amount of bytes to read will be function result - 7. 8 bytes is
/// also fine, as that's the minimal correct frame length.
///
/// * the function will panic if the buffer length is less than 7
///
/// * the function may return wrong result for broken frames
///
/// * the function may return ErrorKind::FrameBroken for broken ASCII frames
pub fn guess_frame_len<'a>(frame: &[u8], proto: ModbusProto) -> Result<u8, ErrorKind> {
    let mut buf: ModbusFrame = [0; 256];
    let f;
    let extra;
    let multiplier;
    match proto {
        ModbusProto::Rtu => {
            f = frame;
            extra = 2;
            multiplier = 1;
        }
        ModbusProto::Ascii => match parse_ascii_frame(&frame, frame.len(), &mut buf, 0) {
            Ok(_) => {
                f = &buf;
                extra = 5;
                multiplier = 2;
            }
            Err(e) => return Err(e),
        },
        ModbusProto::TcpUdp => unimplemented!("unable to guess frame length for TCP/UDP"),
    };
    return match f[1] {
        15 | 16 => Ok((f[6] + 7) * multiplier + extra),
        _ => Ok(6 * multiplier + extra),
    };
}

/// Process Modbus frame
///
/// Simple example of Modbus/UDP blocking server:
///
/// ```rust,ignore
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
/// There's no context param, the function always unlocks Modbus context itself: in a
/// single-threaded environment set "single" feature to use fake mutex, in multi-thread
/// environments frame processing is usually a responsibility of the dedicated "processor" thread.
///
/// For Modbus ASCII, the frame should be parsed first to binary format (parse_ascii_frame
/// function) and the result must be converted to ASCII (generate_ascii_frame).
///
/// The function returns Error in cases:
///
/// * **rmodbus::ErrorKind::FrameBroken**: the frame header is absolutely incorrect and there's no
///   way to form a valid Modbus error reply
///
/// * **rmodbus::ErrorKind::FrameCRCError**: frame CRC error (Modbus RTU, ASCII)
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
        if proto_id != 0 || length < 6 || length > 250 {
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
                || (proto == ModbusProto::Rtu
                    && calc_crc16(frame, $len)
                        == u16::from_le_bytes([frame[$len as usize], frame[$len as usize + 1]]))
                || (proto == ModbusProto::Ascii && calc_lrc(frame, $len) == frame[$len as usize])
        };
    }
    macro_rules! response_error {
        ($err:expr) => {
            match proto {
                ModbusProto::TcpUdp => {
                    if response
                        .add_bulk(&[0, 3, frame[6], frame[7] + 0x80, $err])
                        .is_err()
                    {
                        return Err(ErrorKind::OOB);
                    }
                }
                ModbusProto::Rtu | ModbusProto::Ascii => {
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
            match proto {
                ModbusProto::Rtu => {
                    let crc = calc_crc16(&response.get_slice(), response.get_len() as u8);
                    if response.add_bulk(&crc.to_le_bytes()).is_err() {
                        return Err(ErrorKind::OOB);
                    }
                }
                ModbusProto::Ascii => {
                    let lrc = calc_lrc(&response.get_slice(), response.get_len() as u8);
                    if response.add(lrc).is_err() {
                        return Err(ErrorKind::OOB);
                    }
                }
                _ => {}
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
            Err(ErrorKind::OOBContext) => {
                response.cut_end(5, 0);
                response_error!(0x02);
                finalize_response!();
                Ok(())
            }
            Err(_) => Err(ErrorKind::OOB),
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
            Err(ErrorKind::OOBContext) => {
                response.cut_end(5, 0);
                response_error!(0x02);
                finalize_response!();
                Ok(())
            }
            Err(_) => Err(ErrorKind::OOB),
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
        if !broadcast {
            response_error!(0x01);
            finalize_response!();
        }
        return Ok(());
    }
}
