#![ doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "README.md" ) ) ]
#![ doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "CHANGELOG.md" ) ) ]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod client;
pub mod consts;
pub mod server;

mod vector;
pub use vector::VectorTrait;

mod error;
pub use error::ErrorKind;

#[cfg(test)]
mod tests;

/// Modbus protocol selection for frame processing
///
/// * for **TcpUdp**, Modbus TCP headers are parsed / added to replies
/// * for **Rtu**, frame checksums are verified / added to replies
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ModbusProto {
    Rtu,
    Ascii,
    TcpUdp,
}

/// Standard Modbus frame buffer
///
/// As max length of Modbus frame + headers is always 256 bytes or less, the frame buffer is a
/// fixed [u8; 256] array.
pub type ModbusFrameBuf = [u8; 256];

/// Parse ASCII Modbus frame
///
/// data - input buffer
/// data_len - how many bytes to parse in buffer
/// frame_buf - frame buffer to write output
/// frame_pos - position in frame buffer to write
///
/// The frame can be parsed fully or partially (use frame_pos)
///
/// Returns number of bytes parsed
///
/// Errors:
///
/// * **OOB** input is larger than frame buffer (starting from frame_pos)
/// * **FrameBroken** unable to decode input hex string
pub fn parse_ascii_frame(
    data: &[u8],
    data_len: usize,
    frame_buf: &mut [u8],
    frame_pos: u8,
) -> Result<u8, ErrorKind> {
    let mut data_pos = usize::from(data[0] == 58);
    let mut cpos = frame_pos;
    while data_pos < data_len {
        if cpos == 255 {
            return Err(ErrorKind::OOB);
        }
        let ch = data[data_pos];
        if ch == 10 || ch == 13 || ch == 0 {
            break;
        }
        let c = chr_to_hex(data[data_pos])?;
        data_pos += 1;
        if data_pos >= data_len {
            return Err(ErrorKind::OOB);
        }
        let c2 = chr_to_hex(data[data_pos])?;
        frame_buf[cpos as usize] = c * 0x10 + c2;
        data_pos += 1;
        cpos += 1;
    }
    Ok(cpos - frame_pos)
}

/// Generate ASCII frame
///
/// Generates ASCII frame from binary response, made by "process_frame" function (response must be
/// supplited as slice)
pub fn generate_ascii_frame<V: VectorTrait<u8>>(
    data: &[u8],
    result: &mut V,
) -> Result<(), ErrorKind> {
    result.clear();
    result.push(58)?;
    for d in data {
        result.push(hex_to_chr(d >> 4))?;
        result.push(hex_to_chr(*d & 0xf))?;
    }
    result.push(0x0D)?;
    result.push(0x0A)
}

fn calc_crc16(frame: &[u8], data_length: u8) -> u16 {
    let mut crc: u16 = 0xffff;
    for i in frame.iter().take(data_length as usize) {
        crc ^= u16::from(*i);
        for _ in (0..8).rev() {
            if (crc & 0x0001) == 0 {
                crc >>= 1;
            } else {
                crc >>= 1;
                crc ^= 0xA001;
            }
        }
    }
    crc
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
fn calc_lrc(frame: &[u8], data_length: u8) -> u8 {
    let mut lrc: i32 = 0;
    for i in 0..data_length {
        lrc -= i32::from(frame[i as usize]);
    }
    lrc as u8
}

fn chr_to_hex(c: u8) -> Result<u8, ErrorKind> {
    match c {
        48..=57 => Ok(c - 48),
        65..=70 => Ok(c - 55),
        _ => Err(ErrorKind::FrameBroken),
    }
}

#[inline]
fn hex_to_chr(h: u8) -> u8 {
    if h < 10 {
        h + 48
    } else {
        h + 55
    }
}

/// Guess response frame length
///
/// Frames are often read byte-by-byte. The function allows to guess total frame length, having
/// first 7 (or more) bytes read.
///
/// How to use: read at least first 6 bytes (3 for RTU, 7 for ASCII) into buffer and call the
/// function to guess the total frame length. The remaining amount of bytes to read will be
/// function result - 7. 8 bytes is also fine, as that's the minimal correct frame length.
///
/// * the function may return wrong result for broken frames
///
/// * the function may return ErrorKind::FrameBroken for broken ASCII frames
///
/// # Panics
///
/// The function panics if the buffer length is less than 6 (3 for RTU, 7 for ASCII)
pub fn guess_response_frame_len(buf: &[u8], proto: ModbusProto) -> Result<u8, ErrorKind> {
    let mut b: ModbusFrameBuf = [0; 256];
    let (f, multiplier, extra) = match proto {
        ModbusProto::TcpUdp => {
            let proto = u16::from_be_bytes([buf[2], buf[3]]);
            if proto == 0 {
                let len = u16::from_be_bytes([buf[4], buf[5]]) + 6;
                if len > u16::from(u8::MAX) {
                    return Err(ErrorKind::FrameBroken);
                }
                #[allow(clippy::cast_possible_truncation)]
                return Ok(len as u8);
            }
            return Err(ErrorKind::FrameBroken);
        }
        ModbusProto::Rtu => (buf, 1, 2), // two bytes CRC16
        ModbusProto::Ascii => {
            parse_ascii_frame(buf, buf.len(), &mut b, 0)?;
            (&b[..], 2, 5) // : + two chars LRC + \r\n
        }
    };
    let func = f[1];
    let len: usize = if func < 0x80 {
        match func {
            1..=4 => (f[2] as usize + 3) * multiplier + extra,
            5 | 6 | 15 | 16 => 6 * multiplier + extra,
            _ => {
                return Err(ErrorKind::FrameBroken);
            }
        }
    } else {
        3 * multiplier + extra
    };
    if len > u8::MAX as usize {
        Err(ErrorKind::FrameBroken)
    } else {
        #[allow(clippy::cast_possible_truncation)]
        Ok(len as u8)
    }
}

/// Guess request frame length
///
/// Frames are often read byte-by-byte. The function allows to guess total frame length, having
/// first 7 (or more) bytes read.
///
/// How to use: read at least first 7 bytes (16 for ASCII) into buffer and call the function to
/// guess the total frame length. The remaining amount of bytes to read will be function result -
/// 7. 8 bytes is also fine, as that's the minimal correct frame length.
///
/// * the function may return wrong result for broken frames
///
/// * the function may return ErrorKind::FrameBroken for broken ASCII frames
///
/// # Panics
///
/// The function panics if the buffer length is less than 7 (for ASCII - 16)
pub fn guess_request_frame_len(frame: &[u8], proto: ModbusProto) -> Result<u8, ErrorKind> {
    let mut buf: ModbusFrameBuf = [0; 256];
    let (f, extra, multiplier) = match proto {
        ModbusProto::Rtu => (frame, 2, 1),
        ModbusProto::Ascii => {
            parse_ascii_frame(frame, frame.len(), &mut buf, 0)?;
            (&buf[..], 5, 2)
        }
        ModbusProto::TcpUdp => {
            let proto = u16::from_be_bytes([frame[2], frame[3]]);
            if proto == 0 {
                let len = u16::from_be_bytes([frame[4], frame[5]]) + 6;
                if len > u16::from(u8::MAX) {
                    return Err(ErrorKind::FrameBroken);
                }
                #[allow(clippy::cast_possible_truncation)]
                return Ok(len as u8);
            }
            return Err(ErrorKind::FrameBroken);
        }
    };
    let len: usize = match f[1] {
        15 | 16 => (f[6] as usize + 7) * multiplier + extra,
        _ => 6 * multiplier + extra,
    };
    if len > u8::MAX as usize {
        Err(ErrorKind::FrameBroken)
    } else {
        #[allow(clippy::cast_possible_truncation)]
        Ok(len as u8)
    }
}
