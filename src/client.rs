use crate::{
    calc_crc16, calc_lrc, consts::ModbusFunction, ErrorKind, ModbusFrameBuf, ModbusProto,
    VectorTrait,
};

/// Modbus client generator/processor
///
/// One object can be used for multiple calls
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModbusRequest {
    /// transaction id, (TCP/UDP only), default: 1. To change, set the value manually
    pub tr_id: u16,
    pub unit_id: u8,
    pub func: ModbusFunction,
    pub reg: u16,
    pub count: u16,
    pub proto: ModbusProto,
}

macro_rules! parse_reg {
    ($self: expr, $buf: expr, $result: expr, $t: ty) => {{
        // let (frame_start, frame_end) = $self.parse_response($buf)?;
        let data = $self.parse_slice($buf)?;
        let mut pos = 0;
        while pos < data.len() - 1 {
            let value = <$t>::from_be_bytes([data[pos], data[pos + 1]]);
            if $result.len() >= usize::from($self.count) {
                break;
            }
            $result.push(value)?;
            pos += 2;
        }
    }};
}

macro_rules! parse_reg32 {
    ($self: expr, $buf: expr, $result: expr, $t: ty) => {{
        // let (frame_start, frame_end) = $self.parse_response($buf)?;
        let data = $self.parse_slice($buf)?;
        let mut pos = 0;
        while pos < data.len() - 3 {
            let value =
                <$t>::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
            if $result.len() >= usize::from($self.count) {
                break;
            }
            $result.push(value)?;
            pos += 4;
        }
    }};
}

impl ModbusRequest {
    /// Crate new Modbus client
    pub fn new(unit_id: u8, proto: ModbusProto) -> Self {
        Self {
            tr_id: 1,
            unit_id,
            // default to GetCoils
            func: ModbusFunction::GetCoils,
            reg: 0,
            count: 0,
            proto,
        }
    }

    pub fn new_tcp_udp(unit_id: u8, tr_id: u16) -> Self {
        Self {
            tr_id,
            unit_id,
            // default to GetCoils
            func: ModbusFunction::GetCoils,
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
        self.func = ModbusFunction::GetCoils;
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
        self.func = ModbusFunction::GetDiscretes;
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
        self.func = ModbusFunction::GetHoldings;
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
        self.func = ModbusFunction::GetInputs;
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
        self.func = ModbusFunction::SetCoil;
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
        self.func = ModbusFunction::SetHolding;
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
        self.func = ModbusFunction::SetHoldingsBulk;
        let mut data: ModbusFrameBuf = [0; 256];
        let mut pos = 0;
        for v in values {
            data[pos] = (v >> 8) as u8;
            data[pos + 1] = *v as u8;
            pos += 2;
        }
        self.generate(&data[..values.len() * 2], request)
    }

    /// Generates a Set Holdings Register (modbus function `0x10`) message into `request`.
    ///
    /// If `values.len()` is odd, then the last byte is interpreted as the lower byte of the last register.
    /// Eg, `generate_set_holdings_bulk_from_slice(1200, [0x0A, 0x0B, 0x0C], &mut v)`
    /// writes `0x0A0B` to register `1200` and `0x000C` to register `1201`
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
        self.func = ModbusFunction::SetHoldingsBulk;
        let mut data: ModbusFrameBuf = [0; 256];
        let mut ptr = 0;
        for v in values.chunks(2) {
            let (h, l) = match *v {
                [h, l] => (h, l),
                [l] => (0x00, l), // pad
                _ => unreachable!(),
            };
            data[ptr] = h;
            data[ptr + 1] = l;
            ptr += 2;
        }
        self.generate(&data[..ptr], request)
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
        self.func = ModbusFunction::SetHoldingsBulk;
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
        self.func = ModbusFunction::SetCoilsBulk;
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
        if unit_id != self.unit_id {
            return Err(ErrorKind::FrameBroken);
        }

        // check if received function is valid
        if !ModbusFunction::try_from(buf[frame_start + 1]).is_ok_and(|f| f == self.func) {
            // func-0x80 but some servers respond any shit
            return Err(ErrorKind::from_modbus_error(buf[frame_start + 2]));
        }

        if self.func.is_read() {
            // len is number of words
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
        parse_reg!(self, buf, result, u16);
        Ok(())
    }

    /// Parse response, make sure there's no Modbus error inside, plus parse response data as i16
    /// (getting holdings, inputs)
    ///
    /// The input buffer SHOULD be cut to actual response length
    pub fn parse_i16<V: VectorTrait<i16>>(
        &self,
        buf: &[u8],
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        parse_reg!(self, buf, result, i16);
        Ok(())
    }

    /// Parse response, make sure there's no Modbus error inside, plus parse response data as u32
    /// (getting holdings, inputs)
    ///
    /// The input buffer SHOULD be cut to actual response length
    pub fn parse_u32<V: VectorTrait<u32>>(
        &self,
        buf: &[u8],
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        parse_reg32!(self, buf, result, u32);
        Ok(())
    }

    /// Parse response, make sure there's no Modbus error inside, plus parse response data as i32
    /// (getting holdings, inputs)
    ///
    /// The input buffer SHOULD be cut to actual response length
    pub fn parse_i32<V: VectorTrait<i32>>(
        &self,
        buf: &[u8],
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        parse_reg32!(self, buf, result, i32);
        Ok(())
    }

    /// Parse response, make sure there's no Modbus error inside, plus parse response data as f32
    /// (getting holdings, inputs)
    ///
    /// The input buffer SHOULD be cut to actual response length
    pub fn parse_f32<V: VectorTrait<f32>>(
        &self,
        buf: &[u8],
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        parse_reg32!(self, buf, result, f32);
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

    /// Parses response data as a UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is not correct UTF-8
    #[cfg(feature = "std")]
    pub fn parse_string_utf8(&self, buf: &[u8]) -> Result<String, ErrorKind> {
        let data = self.parse_slice(buf)?;
        match std::str::from_utf8(data) {
            Ok(s) => Ok(s.to_string()),
            Err(_) => Err(ErrorKind::Utf8Error),
        }
    }

    /// Parse response, make sure there's no Modbus error inside
    /// Returns a raw data slice
    ///
    /// The input buffer SHOULD be cut to actual response length
    pub fn parse_slice<'a>(&'a self, buf: &'a [u8]) -> Result<&'a [u8], ErrorKind> {
        let (frame_start, frame_end) = self.parse_response(buf)?;
        if self.func.is_write() {
            Ok(&buf[frame_start + 2..frame_end])
        } else {
            Ok(&buf[frame_start + 3..frame_end])
        }
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
                result.push((b >> i) & 1 == 1)?;
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
                result.push((b >> i) & 1)?;
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
        request.push(self.unit_id)?;
        request.push(self.func.byte())?;
        request.extend(&self.reg.to_be_bytes())?;
        match self.func {
            ModbusFunction::GetCoils
            | ModbusFunction::GetDiscretes
            | ModbusFunction::GetHoldings
            | ModbusFunction::GetInputs => {
                request.extend(&self.count.to_be_bytes())?;
            }
            ModbusFunction::SetCoil | ModbusFunction::SetHolding => {
                request.extend(data)?;
            }
            ModbusFunction::SetCoilsBulk | ModbusFunction::SetHoldingsBulk => {
                request.extend(&self.count.to_be_bytes())?;
                let l = data.len();
                if l > u8::MAX as usize {
                    return Err(ErrorKind::OOB);
                }
                #[allow(clippy::cast_possible_truncation)]
                request.push(l as u8)?;
                request.extend(data)?;
            }
        }
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
        }
        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;

    struct ExpectedParseResults<'a> {
        parse_slice: &'a [u8],
        parse_u16: &'a [u16],
        parse_i16: &'a [i16],
        parse_bool: &'a [bool],
        /// expected output from parse_string
        parse_string: &'a str,
        /// expected output from parse_string_utf8
        parse_string_utf8: Result<&'a str, ErrorKind>,
    }

    struct ExpectedSet<'a> {
        gen: &'a [u8],
        full_response: &'a [u8],
        parsed: Option<ExpectedParseResults<'a>>,
    }

    fn test_func(
        u_id: u8,
        proto: ModbusProto,
        gen_func: fn(&mut ModbusRequest, &mut Vec<u8>) -> Result<(), ErrorKind>,
        exp: ExpectedSet,
    ) {
        let mut req = ModbusRequest::new(u_id, proto);
        let mut msg = Vec::new();
        gen_func(&mut req, &mut msg).unwrap();
        assert_eq!(
            msg, exp.gen,
            "Generated message mismatch: {:02X?} != {:02X?}",
            msg, exp.gen
        );
        req.parse_response(exp.full_response).unwrap();
        req.parse_ok(exp.full_response).unwrap();

        if let Some(p) = exp.parsed {
            let b = req.parse_slice(exp.full_response).unwrap();
            assert_eq!(
                b, p.parse_slice,
                "parse_slice data mismatch: {:02X?} != {:02X?}",
                b, p.parse_slice
            );

            let mut b = Vec::new();
            req.parse_u16(exp.full_response, &mut b).unwrap();
            assert_eq!(
                b, p.parse_u16,
                "parse_u16 data mismatch: {:02X?} != {:02X?}",
                b, p.parse_u16
            );

            let mut b = Vec::new();
            req.parse_i16(exp.full_response, &mut b).unwrap();
            assert_eq!(
                b, p.parse_i16,
                "parse_i16 data mismatch: {:02X?} != {:02X?}",
                b, p.parse_i16
            );

            let mut b = Vec::new();
            req.parse_bool(exp.full_response, &mut b).unwrap();
            assert_eq!(
                b, p.parse_bool,
                "parse_bool data mismatch: {:?} != {:?}",
                b, p.parse_bool
            );

            let mut s = String::new();
            req.parse_string(exp.full_response, &mut s).unwrap();
            assert_eq!(s, p.parse_string, "parse_string data mismatch");

            let s = req.parse_string_utf8(exp.full_response);
            assert_eq!(
                s,
                p.parse_string_utf8.map(ToString::to_string),
                "parse_string_utf8 data mismatch"
            );
        }
    }

    // test cases taken from https://www.modbustools.com/modbus.html
    #[test]
    fn test_rtu_gen_get_coils() {
        test_func(
            0x04,
            ModbusProto::Rtu,
            |req, msg| req.generate_get_coils(0x000A, 0x000D, msg),
            ExpectedSet {
                gen: &[0x04, 0x01, 0x00, 0x0A, 0x00, 0x0D, 0xDD, 0x98],
                full_response: &[0x04, 0x01, 0x02, 0x0A, 0x11, 0xB3, 0x50],
                parsed: Some(ExpectedParseResults {
                    parse_slice: &[0x0A, 0x11],
                    parse_u16: &[0x0A11_u16],
                    parse_i16: &[0x0A11_i16],
                    parse_bool: &[
                        false, true, false, true, false, false, false, false, true, false, false,
                        false, true,
                    ],
                    parse_string: std::str::from_utf8(&[0x0A, 0x11]).unwrap(),
                    parse_string_utf8: Ok(std::str::from_utf8(&[0x0A, 0x11]).unwrap()),
                }),
            },
        );
    }

    #[test]
    fn test_rtu_gen_get_discretes() {
        test_func(
            0x04,
            ModbusProto::Rtu,
            |req, msg| req.generate_get_discretes(0x000A, 0x000D, msg),
            ExpectedSet {
                gen: &[0x04, 0x02, 0x00, 0x0A, 0x00, 0x0D, 0x99, 0x98],
                full_response: &[0x04, 0x02, 0x02, 0x0A, 0x11, 0xB3, 0x14],
                parsed: Some(ExpectedParseResults {
                    parse_slice: &[0x0A, 0x11],
                    parse_u16: &[0x0A11_u16],
                    parse_i16: &[0x0A11_i16],
                    parse_bool: &[
                        false, true, false, true, false, false, false, false, true, false, false,
                        false, true,
                    ],
                    parse_string: std::str::from_utf8(&[0x0A, 0x11]).unwrap(),
                    parse_string_utf8: Ok(std::str::from_utf8(&[0x0A, 0x11]).unwrap()),
                }),
            },
        );
    }

    #[test]
    fn test_rtu_gen_get_holdings() {
        test_func(
            0x01,
            ModbusProto::Rtu,
            |req, msg| req.generate_get_holdings(0x0000, 0x0002, msg),
            ExpectedSet {
                gen: &[0x01, 0x03, 0x00, 0x00, 0x00, 0x02, 0xC4, 0x0B],
                full_response: &[0x01, 0x03, 0x04, 0x00, 0x06, 0x00, 0x05, 0xDA, 0x31],
                parsed: Some(ExpectedParseResults {
                    parse_slice: &[0x00, 0x06, 0x00, 0x05],
                    parse_u16: &[0x0006_u16, 0x0005_u16],
                    parse_i16: &[0x0006_i16, 0x0005_i16],
                    parse_bool: &[false, false],
                    // it stops at the first null byte... is this a bug?
                    parse_string: std::str::from_utf8(&[]).unwrap(),
                    parse_string_utf8: Ok(std::str::from_utf8(&[0x00, 0x06, 0x00, 0x05]).unwrap()),
                }),
            },
        );
    }

    #[test]
    fn test_rtu_gen_get_inputs() {
        test_func(
            0x01,
            ModbusProto::Rtu,
            |req, msg| req.generate_get_inputs(0x0000, 0x0002, msg),
            ExpectedSet {
                gen: &[0x01, 0x04, 0x00, 0x00, 0x00, 0x02, 0x71, 0xCB],
                full_response: &[0x01, 0x04, 0x04, 0x00, 0x06, 0x00, 0x05, 0xDB, 0x86],
                parsed: Some(ExpectedParseResults {
                    parse_slice: &[0x00, 0x06, 0x00, 0x05],
                    parse_u16: &[0x0006_u16, 0x0005_u16],
                    parse_i16: &[0x0006_i16, 0x0005_i16],
                    parse_bool: &[false, false],
                    // it stops at the first null byte... is this a bug?
                    parse_string: std::str::from_utf8(&[]).unwrap(),
                    parse_string_utf8: Ok(std::str::from_utf8(&[0x00, 0x06, 0x00, 0x05]).unwrap()),
                }),
            },
        );
    }

    #[test]
    fn test_rtu_set_coil() {
        test_func(
            0x11,
            ModbusProto::Rtu,
            |req, msg| req.generate_set_coil(0x00AC, true, msg),
            ExpectedSet {
                gen: &[0x11, 0x05, 0x00, 0xAC, 0xFF, 0x00, 0x4E, 0x8B],
                // write message should mirror the generated message
                full_response: &[0x11, 0x05, 0x00, 0xAC, 0xFF, 0x00, 0x4E, 0x8B],
                parsed: None,
            },
        );
    }

    #[test]
    fn test_rtu_set_holding() {
        test_func(
            0x11,
            ModbusProto::Rtu,
            |req, msg| req.generate_set_holding(0x0001, 0x0003, msg),
            ExpectedSet {
                gen: &[0x11, 0x06, 0x00, 0x01, 0x00, 0x03, 0x9A, 0x9B],
                // write message should mirror the generated message
                full_response: &[0x11, 0x06, 0x00, 0x01, 0x00, 0x03, 0x9A, 0x9B],
                parsed: None,
            },
        );
    }

    // set coils is bugged and input is confusing
    // it should be possible to set not set a coil, eg [0xCD, 0x01] only settings 10 coils instead of 16.
    #[test]
    fn test_rtu_set_coils_bulk() {
        test_func(
            0x11,
            ModbusProto::Rtu,
            |req, msg| {
                req.generate_set_coils_bulk(
                    0x0013,
                    &[1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1],
                    msg,
                )
            },
            ExpectedSet {
                // gen: &[0x11, 0x0F, 0x00, 0x13, 0x00, 0x0A, 0x02, 0xCD, 0x01, 0xBF, 0x0B],
                gen: &[
                    0x11, 0x0F, 0x00, 0x13, 0x00, 0x10, 0x02, 0xB3, 0x80, 0x59, 0xD3,
                ],
                // write message should mirror the generated message
                full_response: &[
                    0x11, 0x0F, 0x00, 0x13, 0x00, 0x10, 0x02, 0xB3, 0x80, 0x59, 0xD3,
                ],
                parsed: None,
            },
        );
    }

    #[test]
    fn test_rtu_set_holdings_bulk() {
        test_func(
            0x11,
            ModbusProto::Rtu,
            |req, msg| req.generate_set_holdings_bulk(0x0001, &[0x000A, 0x0102], msg),
            ExpectedSet {
                gen: &[
                    0x11, 0x10, 0x00, 0x01, 0x00, 0x02, 0x04, 0x00, 0x0A, 0x01, 0x02, 0xC6, 0xF0,
                ],
                // write message should mirror the generated message
                full_response: &[
                    0x11, 0x10, 0x00, 0x01, 0x00, 0x02, 0x04, 0x00, 0x0A, 0x01, 0x02, 0xC6, 0xF0,
                ],
                parsed: None,
            },
        );
    }

    #[test]
    fn test_rtu_set_holdings_bulk_from_slice() {
        test_func(
            0x11,
            ModbusProto::Rtu,
            |req, msg| {
                req.generate_set_holdings_bulk_from_slice(0x0001, &[0x00, 0x0A, 0x01, 0x02], msg)
            },
            ExpectedSet {
                gen: &[
                    0x11, 0x10, 0x00, 0x01, 0x00, 0x02, 0x04, 0x00, 0x0A, 0x01, 0x02, 0xC6, 0xF0,
                ],
                // write message should mirror the generated message
                full_response: &[0x11, 0x10, 0x00, 0x01, 0x00, 0x02, 0x12, 0x98],
                parsed: None,
            },
        );
    }

    /// Odd number of bytes should pad
    #[test]
    fn test_rtu_set_holdings_bulk_from_slice_odd() {
        test_func(
            0x11,
            ModbusProto::Rtu,
            |req, msg| {
                req.generate_set_holdings_bulk_from_slice(
                    0x0001,
                    &[0x00, 0x0A, 0x01, 0x02, 0x03],
                    msg,
                )
            },
            ExpectedSet {
                gen: &[
                    0x11, 0x10, 0x00, 0x01, 0x00, 0x03, 0x06, 0x00, 0x0A, 0x01, 0x02, 0x00, 0x03,
                    0xF1, 0xE9,
                ],
                full_response: &[0x11, 0x10, 0x00, 0x01, 0x00, 0x02, 0x06, 0x98, 0x0F],
                parsed: None,
            },
        );
    }
}
