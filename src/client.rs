// TODO set coil, set holding, set coils bulk, set holdings bulk
// TODO tests
// TODO client examples

use super::*;

pub struct ModbusRequest {
    pub tr_id: u16,
    pub unit_id: u8,
    pub func: u8,
    pub reg: u16,
    pub count: u16,
    pub proto: ModbusProto,
}

impl ModbusRequest {
    pub fn new(unit_id: u8, proto: ModbusProto) -> Self {
        Self {
            tr_id: 1,
            unit_id: unit_id,
            func: 0,
            reg: 0,
            count: 0,
            proto: proto,
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
        return self.generate(&[], request);
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
        return self.generate(&[], request);
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
        return self.generate(&[], request);
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
        return self.generate(&[], request);
    }

    fn parse_response(&self, buf: &[u8]) -> Result<(usize, usize), ErrorKind> {
        let (frame_start, frame_end) = match self.proto {
            ModbusProto::TcpUdp => {
                if buf.len() < 11 {
                    return Err(ErrorKind::FrameBroken);
                }
                let tr_id = u16::from_be_bytes([buf[0], buf[1]]);
                let proto = u16::from_be_bytes([buf[2], buf[3]]);
                if tr_id != self.tr_id || proto != 0 {
                    return Err(ErrorKind::FrameBroken);
                }
                (6, buf.len())
            }
            ModbusProto::Rtu => {
                if buf.len() < 7 {
                    return Err(ErrorKind::FrameBroken);
                }
                let len = buf.len();
                let crc = calc_crc16(buf, len as u8 - 2);
                if crc != u16::from_le_bytes([buf[len - 2], buf[len - 1]]) {
                    return Err(ErrorKind::FrameCRCError);
                }
                (0, buf.len() - 2)
            }
            ModbusProto::Ascii => {
                if buf.len() < 6 {
                    return Err(ErrorKind::FrameBroken);
                }
                let len = buf.len();
                let lrc = calc_lrc(buf, len as u8 - 1);
                if lrc != buf[len - 1] {
                    return Err(ErrorKind::FrameCRCError);
                }
                (0, buf.len() - 1)
            }
        };
        let unit_id = buf[frame_start];
        let func = buf[frame_start + 1];
        if unit_id != self.unit_id || (func != self.func && func < 0x81) {
            return Err(ErrorKind::FrameBroken);
        }
        if func > 0x80 {
            return Err(ErrorKind::from_modbus_error(func - 0x80));
        }
        let len = buf[frame_start + 2] as usize;
        if len * 2 < (frame_end - frame_start) - 3 {
            return Err(ErrorKind::FrameBroken);
        }
        return Ok((frame_start, frame_end));
    }

    pub fn ok(&self, buf: &[u8]) -> Result<(), ErrorKind> {
        match self.parse_response(buf) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(e),
        };
    }

    pub fn parse_as_u16<V: VectorTrait<u16>>(
        &self,
        buf: &[u8],
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        let (frame_start, frame_end) = match self.parse_response(buf) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let mut pos = frame_start + 3;
        while pos < frame_end - 1 {
            let value = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
            if result.get_len() >= self.count as usize {
                break;
            }
            if result.add(value).is_err() {
                return Err(ErrorKind::OOB);
            }
            pos = pos + 2;
        }
        Ok(())
    }

    pub fn parse_as_bool<V: VectorTrait<bool>>(
        &self,
        buf: &[u8],
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        let (frame_start, frame_end) = match self.parse_response(buf) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        for pos in frame_start + 3..frame_end {
            let b = buf[pos];
            for i in 0..8 {
                if result.get_len() >= self.count as usize {
                    break;
                }
                if result.add(b >> i & 1 == 1).is_err() {
                    return Err(ErrorKind::OOB);
                }
            }
        }
        Ok(())
    }

    fn generate<V: VectorTrait<u8>>(&self, data: &[u8], request: &mut V) -> Result<(), ErrorKind> {
        request.clear_all();
        if self.proto == ModbusProto::TcpUdp {
            if request.add_bulk(&self.tr_id.to_be_bytes()).is_err() {
                return Err(ErrorKind::OOB);
            }
            if request.add_bulk(&[0u8, 0, 0, 0]).is_err() {
                return Err(ErrorKind::OOB);
            }
        }
        if request.add_bulk(&[self.unit_id, self.func]).is_err() {
            return Err(ErrorKind::OOB);
        }
        match self.func {
            MODBUS_GET_COILS | MODBUS_GET_DISCRETES | MODBUS_GET_HOLDINGS | MODBUS_GET_INPUTS => {
                if request.add_bulk(&self.reg.to_be_bytes()).is_err() {
                    return Err(ErrorKind::OOB);
                }
                if request.add_bulk(&self.count.to_be_bytes()).is_err() {
                    return Err(ErrorKind::OOB);
                }
            }
            _ => unimplemented!(),
        };
        match self.proto {
            ModbusProto::TcpUdp => {
                let len = ((request.get_len() as u16) - 6).to_be_bytes();
                request.replace(4, len[0]);
                request.replace(5, len[1]);
            }
            ModbusProto::Rtu => {
                let crc = calc_crc16(request.get_slice(), request.get_len() as u8);
                if request.add_bulk(&crc.to_le_bytes()).is_err() {
                    return Err(ErrorKind::OOB);
                }
            }
            ModbusProto::Ascii => {
                let lrc = calc_lrc(request.get_slice(), request.get_len() as u8);
                if request.add(lrc).is_err() {
                    return Err(ErrorKind::OOB);
                }
            }
        };
        Ok(())
    }
}
