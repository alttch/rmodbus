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

#[cfg(not(any(feature = "tcpudp", feature = "rtu", feature = "ascii",)))]
compile_error!("At least one features [tcpudp, rtu, ascii] must be enabled");

const ASCII_TERMINATION_LEN: usize = 5;
// two bytes CRC16
const RTU_TERMINATION_LEN: usize = 2;
const ASCII_MULTIPLIER: usize = 2;
const RTU_MULTIPLIER: usize = 1;

/// Modbus protocol selection for frame processing
///
/// * for **TcpUdp**, Modbus TCP headers are parsed / added to replies
/// * for **Rtu**, frame checksums are verified / added to replies
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ModbusProto {
    #[cfg(feature = "rtu")]
    Rtu,
    #[cfg(feature = "ascii")]
    Ascii,
    #[cfg(feature = "tcpudp")]
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
#[cfg(feature = "ascii")]
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
#[cfg(feature = "ascii")]
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

#[cfg(feature = "rtu")]
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

#[cfg(feature = "ascii")]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
fn calc_lrc(frame: &[u8], data_length: u8) -> u8 {
    let mut lrc: i32 = 0;
    for i in 0..data_length {
        lrc -= i32::from(frame[i as usize]);
    }
    lrc as u8
}

#[cfg(feature = "ascii")]
fn chr_to_hex(c: u8) -> Result<u8, ErrorKind> {
    match c {
        48..=57 => Ok(c - 48),
        65..=70 => Ok(c - 55),
        _ => Err(ErrorKind::FrameBroken),
    }
}

#[cfg(feature = "ascii")]
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
/// Frames are often read byte-by-byte. The function lets a caller determine the total frame
/// length after reading only the first few bytes, so the rest can be read in one shot.
///
/// Minimum buffer per protocol:
///
/// * **TcpUdp**: 6 bytes (the MBAP header fully describes the frame length, including
///   exception responses).
/// * **Rtu**: 2 bytes to detect an exception response (`func & 0x80 != 0`), which is always
///   5 bytes on the wire. For a non-exception response, 3 bytes are required so the byte
///   count / register count byte can be inspected.
/// * **Ascii**: at least 5 ASCII chars (`:AAFF`) to detect an exception (always 11 chars
///   total), or 7 chars (`:AAFFBB`) for a non-exception response. `parse_ascii_frame` stops
///   decoding at the first `\r`, `\n`, or `\0`.
///
/// If fewer bytes are supplied than required for the detected frame kind, `ErrorKind::OOB`
/// is returned instead of panicking. `ErrorKind::FrameBroken` is returned for malformed
/// frames (unknown function code, ASCII decode failure, or computed length > 255).
pub fn guess_response_frame_len(buf: &[u8], proto: ModbusProto) -> Result<u8, ErrorKind> {
    match proto {
        #[cfg(feature = "tcpudp")]
        ModbusProto::TcpUdp => guess_tcp_response_frame_len(buf),
        #[cfg(feature = "rtu")]
        ModbusProto::Rtu => {
            guess_serial_response_frame_len(buf, buf.len(), RTU_MULTIPLIER, RTU_TERMINATION_LEN)
        }
        #[cfg(feature = "ascii")]
        ModbusProto::Ascii => {
            if buf.is_empty() {
                return Err(ErrorKind::OOB);
            }
            let mut b: ModbusFrameBuf = [0; 256];
            let n = parse_ascii_frame(buf, buf.len(), &mut b, 0)?;
            guess_serial_response_frame_len(
                &b[..],
                usize::from(n),
                ASCII_MULTIPLIER,
                ASCII_TERMINATION_LEN,
            )
        }
    }
}
#[cfg(feature = "tcpudp")]
fn guess_tcp_response_frame_len(buf: &[u8]) -> Result<u8, ErrorKind> {
    if buf.len() < 6 {
        return Err(ErrorKind::OOB);
    }
    let proto = u16::from_be_bytes([buf[2], buf[3]]);
    if proto == 0 {
        let len = u32::from(u16::from_be_bytes([buf[4], buf[5]])) + 6;
        if len > u32::from(u8::MAX) {
            return Err(ErrorKind::FrameBroken);
        }
        #[allow(clippy::cast_possible_truncation)]
        return Ok(len as u8);
    }
    Err(ErrorKind::FrameBroken)
}

#[cfg(any(feature = "rtu", feature = "ascii"))]
fn guess_serial_response_frame_len(
    frame: &[u8],
    frame_len: usize,
    multiplier: usize,
    extra: usize,
) -> Result<u8, ErrorKind> {
    if frame_len < 2 {
        return Err(ErrorKind::OOB);
    }
    let func = frame[1];
    let len: usize = if func < 0x80 {
        if frame_len < 3 {
            return Err(ErrorKind::OOB);
        }
        match func {
            1..=4 => (frame[2] as usize + 3) * multiplier + extra,
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
/// Frames are often read byte-by-byte. The function lets a caller determine the total frame
/// length after reading only the first few bytes, so the rest can be read in one shot.
///
/// Minimum buffer per protocol:
///
/// * **TcpUdp**: 6 bytes (the MBAP header fully describes the frame length).
/// * **Rtu**: 7 bytes for write-multiple functions (15/16) so the byte-count byte can be
///   inspected; 2 bytes (`unit_id` + `func`) are sufficient for all other functions.
/// * **Ascii**: at least 15 ASCII chars for write-multiple (15/16), or 5 chars (`:AAFF`)
///   for all other functions. `parse_ascii_frame` stops decoding at the first `\r`, `\n`,
///   or `\0`.
///
/// If fewer bytes are supplied than required for the detected frame kind, `ErrorKind::OOB`
/// is returned instead of panicking. `ErrorKind::FrameBroken` is returned for malformed
/// frames (ASCII decode failure or computed length > 255).
pub fn guess_request_frame_len(frame: &[u8], proto: ModbusProto) -> Result<u8, ErrorKind> {
    match proto {
        #[cfg(feature = "tcpudp")]
        ModbusProto::TcpUdp => guess_tcp_request_frame_len(frame),
        #[cfg(feature = "rtu")]
        ModbusProto::Rtu => {
            guess_serial_request_frame_len(frame, frame.len(), RTU_MULTIPLIER, RTU_TERMINATION_LEN)
        }
        #[cfg(feature = "ascii")]
        ModbusProto::Ascii => {
            if frame.is_empty() {
                return Err(ErrorKind::OOB);
            }
            let mut buf: ModbusFrameBuf = [0; 256];
            let n = parse_ascii_frame(frame, frame.len(), &mut buf, 0)?;
            guess_serial_request_frame_len(
                &buf[..],
                usize::from(n),
                ASCII_MULTIPLIER,
                ASCII_TERMINATION_LEN,
            )
        }
    }
}

#[cfg(any(feature = "rtu", feature = "ascii"))]
#[inline]
fn guess_serial_request_frame_len(
    frame: &[u8],
    frame_len: usize,
    multiplier: usize,
    extra: usize,
) -> Result<u8, ErrorKind> {
    if frame_len < 2 {
        return Err(ErrorKind::OOB);
    }
    let len: usize = match frame[1] {
        15 | 16 => {
            if frame_len < 7 {
                return Err(ErrorKind::OOB);
            }
            (frame[6] as usize + 7) * multiplier + extra
        }
        _ => 6 * multiplier + extra,
    };
    if len > u8::MAX as usize {
        Err(ErrorKind::FrameBroken)
    } else {
        #[allow(clippy::cast_possible_truncation)]
        Ok(len as u8)
    }
}

#[cfg(feature = "tcpudp")]
#[inline]
fn guess_tcp_request_frame_len(frame: &[u8]) -> Result<u8, ErrorKind> {
    if frame.len() < 6 {
        return Err(ErrorKind::OOB);
    }
    let proto = u16::from_be_bytes([frame[2], frame[3]]);
    if proto == 0 {
        let len = u32::from(u16::from_be_bytes([frame[4], frame[5]])) + 6;
        if len > u32::from(u8::MAX) {
            return Err(ErrorKind::FrameBroken);
        }
        #[allow(clippy::cast_possible_truncation)]
        return Ok(len as u8);
    }
    Err(ErrorKind::FrameBroken)
}
