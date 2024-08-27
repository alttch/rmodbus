use crate::consts::{
    MODBUS_GET_COILS, MODBUS_GET_DISCRETES, MODBUS_GET_HOLDINGS, MODBUS_GET_INPUTS,
    MODBUS_SET_COIL, MODBUS_SET_COILS_BULK, MODBUS_SET_HOLDING, MODBUS_SET_HOLDINGS_BULK,
};
use crate::{calc_crc16, calc_lrc, ErrorKind, ModbusFrameBuf, ModbusProto, VectorTrait};

/// Modbus client generator/processor
///
/// One object can be used for multiple calls
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModbusRequest {
    /// transaction id, (TCP/UDP only), default: 1. To change, set the value manually
    pub tr_id: u16,
    pub unit_id: u8,
    pub func: u8,
    pub reg: u16,
    pub count: u16,
    pub proto: ModbusProto,
}

impl ModbusRequest {
    /// Crate new Modbus client
    pub fn new(unit_id: u8, proto: ModbusProto) -> Self {
        Self {
            tr_id: 1,
            unit_id,
            func: 0,
            reg: 0,
            count: 0,
            proto,
        }
    }

    pub fn new_tcp_udp(unit_id: u8, tr_id: u16) -> Self {
        Self {
            tr_id,
            unit_id,
            func: 0,
            reg: 0,
            count: 0,
            proto: ModbusProto::TcpUdp,
        }
    }

    pub fn generate_get_coils<V: VectorTrait<u8>>(
        &mut self,
        reg: u16,
        count: u16,
        request: &mut V,
    ) -> Result<(), ErrorKind> {
        self.reg = reg;
        self.count = count;
        self.func = MODBUS_GET_COILS;
        self.generate(&[], request)
    }

    pub fn generate_get_discretes<V: VectorTrait<u8>>(
        &mut self,
        reg: u16,
        count: u16,
        request: &mut V,
    ) -> Result<(), ErrorKind> {
        self.reg = reg;
        self.count = count;
        self.func = MODBUS_GET_DISCRETES;
        self.generate(&[], request)
    }

    pub fn generate_get_holdings<V: VectorTrait<u8>>(
        &mut self,
        reg: u16,
        count: u16,
        request: &mut V,
    ) -> Result<(), ErrorKind> {
        self.reg = reg;
        self.count = count;
        self.func = MODBUS_GET_HOLDINGS;
        self.generate(&[], request)
    }

    pub fn generate_get_inputs<V: VectorTrait<u8>>(
        &mut self,
        reg: u16,
        count: u16,
        request: &mut V,
    ) -> Result<(), ErrorKind> {
        self.reg = reg;
        self.count = count;
        self.func = MODBUS_GET_INPUTS;
        self.generate(&[], request)
    }

    /// a value can be u8 or bool
    pub fn generate_set_coil<V: VectorTrait<u8>, S: Into<u8>>(
        &mut self,
        reg: u16,
        value: S,
        request: &mut V,
    ) -> Result<(), ErrorKind> {
        self.reg = reg;
        self.count = 1;
        self.func = MODBUS_SET_COIL;
        self.generate(
            &[
                if Into::<u8>::into(value) > 0 {
                    0xff
                } else {
                    0x00
                },
                0x00,
            ],
            request,
        )
    }

    pub fn generate_set_holding<V: VectorTrait<u8>>(
        &mut self,
        reg: u16,
        value: u16,
        request: &mut V,
    ) -> Result<(), ErrorKind> {
        self.reg = reg;
        self.count = 1;
        self.func = MODBUS_SET_HOLDING;
        self.generate(&value.to_be_bytes(), request)
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn generate_set_holdings_bulk<V: VectorTrait<u8>>(
        &mut self,
        reg: u16,
        values: &[u16],
        request: &mut V,
    ) -> Result<(), ErrorKind> {
        if values.len() > 125 {
            return Err(ErrorKind::OOB);
        }
        self.reg = reg;
        self.count = u16::try_from(values.len())?;
        self.func = MODBUS_SET_HOLDINGS_BULK;
        let mut data: ModbusFrameBuf = [0; 256];
        let mut pos = 0;
        for v in values {
            data[pos] = (v >> 8) as u8;
            data[pos + 1] = *v as u8;
            pos += 2;
        }
        self.generate(&data[..values.len() * 2], request)
    }

    pub fn generate_set_holdings_bulk_from_slice<V: VectorTrait<u8>>(
        &mut self,
        reg: u16,
        values: &[u8],
        request: &mut V,
    ) -> Result<(), ErrorKind> {
        if values.len() > 125 {
            return Err(ErrorKind::OOB);
        }
        self.reg = reg;
        self.count = u16::try_from((values.len() + 1) / 2)?; // count is number of u16's
        self.func = MODBUS_SET_HOLDINGS_BULK;
        let mut data: ModbusFrameBuf = [0; 256];
        for (i, v) in values.iter().enumerate() {
            data[i] = *v;
        }
        self.generate(&data[..values.len()], request)
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn generate_set_holdings_string<V: VectorTrait<u8>>(
        &mut self,
        reg: u16,
        values: &str,
        request: &mut V,
    ) -> Result<(), ErrorKind> {
        let values = values.as_bytes();
        let length = values.len() + values.len() % 2;
        if length > 250 {
            return Err(ErrorKind::OOB);
        }
        self.reg = reg;
        self.count = length as u16 / 2u16;
        self.func = MODBUS_SET_HOLDINGS_BULK;
        let mut data: ModbusFrameBuf = [0; 256];
        for (pos, v) in values.iter().enumerate() {
            data[pos] = *v;
        }
        self.generate(&data[..length], request)
    }

    /// values can be u8 or bool
    #[allow(clippy::cast_possible_truncation)]
    pub fn generate_set_coils_bulk<V: VectorTrait<u8>, S: Into<u8> + Copy>(
        &mut self,
        reg: u16,
        values: &[S],
        request: &mut V,
    ) -> Result<(), ErrorKind> {
        let l = values.len();
        if l > 4000 {
            return Err(ErrorKind::OOB);
        }
        self.reg = reg;
        self.count = l as u16;
        self.func = MODBUS_SET_COILS_BULK;
        let mut data: ModbusFrameBuf = [0; 256];
        let mut pos = 0;
        let mut cbyte = 0;
        let mut bidx = 0;
        for v in values {
            if Into::<u8>::into(*v) > 0 {
                cbyte |= 1 << bidx;
            }
            bidx += 1;
            if bidx > 7 {
                bidx = 0;
                data[pos] = cbyte;
                pos += 1;
                cbyte = 0;
            }
        }
        let len;
        if bidx > 0 {
            data[pos] = cbyte;
            len = pos + 1;
        } else {
            len = pos;
        }
        self.generate(&data[..len], request)
    }

    fn parse_response(&self, buf: &[u8]) -> Result<(usize, usize), ErrorKind> {
        let (frame_start, frame_end) = match self.proto {
            ModbusProto::TcpUdp => {
                let l = buf.len();
                if l < 9 {
                    return Err(ErrorKind::FrameBroken);
                }
                let tr_id = u16::from_be_bytes([buf[0], buf[1]]);
                let proto = u16::from_be_bytes([buf[2], buf[3]]);
                if tr_id != self.tr_id || proto != 0 {
                    return Err(ErrorKind::FrameBroken);
                }
                (6, l)
            }
            ModbusProto::Rtu => {
                let mut l = buf.len();
                if l < 5 {
                    return Err(ErrorKind::FrameBroken);
                }
                l -= 2;
                if l > u8::MAX as usize {
                    return Err(ErrorKind::FrameBroken);
                }
                #[allow(clippy::cast_possible_truncation)]
                let crc = calc_crc16(buf, l as u8);
                if crc != u16::from_le_bytes([buf[l], buf[l + 1]]) {
                    return Err(ErrorKind::FrameCRCError);
                }
                (0, l)
            }
            ModbusProto::Ascii => {
                let mut l = buf.len();
                if l < 4 {
                    return Err(ErrorKind::FrameBroken);
                }
                l -= 1;
                if l > u8::MAX as usize {
                    return Err(ErrorKind::FrameBroken);
                }
                #[allow(clippy::cast_possible_truncation)]
                let lrc = calc_lrc(buf, l as u8);
                if lrc != buf[l] {
                    return Err(ErrorKind::FrameCRCError);
                }
                (0, l)
            }
        };
        let unit_id = buf[frame_start];
        let func = buf[frame_start + 1];
        if unit_id != self.unit_id {
            return Err(ErrorKind::FrameBroken);
        }
        if func != self.func {
            // func-0x80 but some servers respond any shit
            return Err(ErrorKind::from_modbus_error(buf[frame_start + 2]));
        }
        if self.func > 0 && self.func < 5 {
            let len = buf[frame_start + 2] as usize;
            if len * 2 < (frame_end - frame_start) - 3 {
                return Err(ErrorKind::FrameBroken);
            }
        }
        Ok((frame_start, frame_end))
    }

    /// Parse response and make sure there's no Modbus error inside
    ///
    /// The input buffer SHOULD be cut to actual response length
    pub fn parse_ok(&self, buf: &[u8]) -> Result<(), ErrorKind> {
        self.parse_response(buf)?;
        Ok(())
    }

    /// Parse response, make sure there's no Modbus error inside, plus parse response data as u16
    /// (getting holdings, inputs)
    ///
    /// The input buffer SHOULD be cut to actual response length
    pub fn parse_u16<V: VectorTrait<u16>>(
        &self,
        buf: &[u8],
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        let (frame_start, frame_end) = self.parse_response(buf)?;
        let mut pos = frame_start + 3;
        while pos < frame_end - 1 {
            let value = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
            if result.len() >= self.count as usize {
                break;
            }
            result.push(value)?;
            pos += 2;
        }
        Ok(())
    }

    /// Parse response, make sure there's no Modbus error inside, plus parse response data as u16
    /// (getting holdings, inputs)
    ///
    /// The input buffer SHOULD be cut to actual response length
    #[cfg(feature = "std")]
    pub fn parse_string(&self, buf: &[u8], result: &mut String) -> Result<(), ErrorKind> {
        let (frame_start, frame_end) = self.parse_response(buf)?;
        let val = &buf[frame_start + 3..frame_end];
        let vl = val.iter().position(|&c| c == b'\0').unwrap_or(val.len());
        *result = match std::str::from_utf8(&val[..vl]) {
            Ok(v) => v.to_string(),
            Err(_) => return Err(ErrorKind::Utf8Error),
        };
        Ok(())
    }

    /// Parse response, make sure there's no Modbus error inside
    /// Returns a raw data slice
    ///
    /// The input buffer SHOULD be cut to actual response length
    pub fn parse_slice<'a>(&'a self, buf: &'a [u8]) -> Result<&[u8], ErrorKind> {
        let (frame_start, frame_end) = self.parse_response(buf)?;
        let val = match self.func {
            MODBUS_SET_COIL
            | MODBUS_SET_COILS_BULK
            | MODBUS_SET_HOLDING
            | MODBUS_SET_HOLDINGS_BULK => {
                // no data bytes count byte -> skip 1 fewer byte
                &buf[frame_start + 2..frame_end]
            }
            _ => &buf[frame_start + 3..frame_end],
        };
        Ok(val)
    }

    /// Parse response, make sure there's no Modbus error inside, plus parse response data as bools
    /// (getting coils, discretes)
    ///
    /// The input buffer SHOULD be cut to actual response length
    pub fn parse_bool<V: VectorTrait<bool>>(
        &self,
        buf: &[u8],
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        let (frame_start, frame_end) = self.parse_response(buf)?;
        for b in buf.iter().take(frame_end).skip(frame_start + 3) {
            for i in 0..8 {
                if result.len() >= self.count as usize {
                    break;
                }
                result.push(b >> i & 1 == 1)?;
            }
        }
        Ok(())
    }

    /// Parse response, make sure there's no Modbus error inside, plus parse response data as bools
    /// represented as u8 (getting coils, discretes)
    ///
    /// The input buffer SHOULD be cut to actual response length
    pub fn parse_bool_u8<V: VectorTrait<u8>>(
        &self,
        buf: &[u8],
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        let (frame_start, frame_end) = self.parse_response(buf)?;
        for b in buf.iter().take(frame_end).skip(frame_start + 3) {
            for i in 0..8 {
                if result.len() >= self.count as usize {
                    break;
                }
                result.push(b >> i & 1)?;
            }
        }
        Ok(())
    }

    fn generate<V: VectorTrait<u8>>(&self, data: &[u8], request: &mut V) -> Result<(), ErrorKind> {
        request.clear();
        if self.proto == ModbusProto::TcpUdp {
            request.extend(&self.tr_id.to_be_bytes())?;
            request.extend(&[0u8, 0, 0, 0])?;
        }
        request.extend(&[self.unit_id, self.func])?;
        request.extend(&self.reg.to_be_bytes())?;
        match self.func {
            MODBUS_GET_COILS | MODBUS_GET_DISCRETES | MODBUS_GET_HOLDINGS | MODBUS_GET_INPUTS => {
                request.extend(&self.count.to_be_bytes())?;
            }
            MODBUS_SET_COIL | MODBUS_SET_HOLDING => {
                for v in data {
                    request.push(*v)?;
                }
            }
            MODBUS_SET_COILS_BULK | MODBUS_SET_HOLDINGS_BULK => {
                request.extend(&self.count.to_be_bytes())?;
                let l = data.len();
                if l > u8::MAX as usize {
                    return Err(ErrorKind::OOB);
                }
                #[allow(clippy::cast_possible_truncation)]
                request.push(l as u8)?;
                for v in data {
                    request.push(*v)?;
                }
            }
            _ => unimplemented!(),
        };
        match self.proto {
            ModbusProto::TcpUdp => {
                let mut l = request.len();
                if l < 6 {
                    return Err(ErrorKind::OOB);
                }
                l -= 6;
                if l > u16::MAX as usize {
                    return Err(ErrorKind::OOB);
                }
                #[allow(clippy::cast_possible_truncation)]
                let len_buf = (l as u16).to_be_bytes();
                request.replace(4, len_buf[0]);
                request.replace(5, len_buf[1]);
            }
            ModbusProto::Rtu => {
                let l = request.len();
                if l > u8::MAX as usize {
                    return Err(ErrorKind::OOB);
                }
                #[allow(clippy::cast_possible_truncation)]
                let crc = calc_crc16(request.as_slice(), l as u8);
                request.extend(&crc.to_le_bytes())?;
            }
            ModbusProto::Ascii => {
                let l = request.len();
                if l > u8::MAX as usize {
                    return Err(ErrorKind::OOB);
                }
                #[allow(clippy::cast_possible_truncation)]
                let lrc = calc_lrc(request.as_slice(), l as u8);
                request.push(lrc)?;
            }
        };
        Ok(())
    }
}
