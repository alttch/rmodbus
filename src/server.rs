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
/// How to use: read at least first 7 bytes (16 for ASCII) into buffer and call the function to
/// guess the total frame length. The remaining amount of bytes to read will be function result -
/// 7. 8 bytes is also fine, as that's the minimal correct frame length.
///
/// * the function will panic if the buffer length is less than 7 (for ASCII - 16)
///
/// * the function may return wrong result for broken frames
///
/// * the function may return ErrorKind::FrameBroken for broken ASCII frames
pub fn guess_frame_len(frame: &[u8], proto: ModbusProto) -> Result<u8, ErrorKind> {
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

