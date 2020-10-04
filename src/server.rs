#[path = "context.rs"]
pub mod context;

use super::*;

macro_rules! tcp_response_set_data_len {
    ($self: expr, $len:expr) => {
        if $self.proto == ModbusProto::TcpUdp {
            if $self
                .response
                .add_bulk(&($len as u16).to_be_bytes())
                .is_err()
            {
                return Err(ErrorKind::OOB);
            }
        }
    };
}

/// Modbus frame processor
///
/// ```rust, no_run
/// use rmodbus::{ModbusFrameBuf, ModbusProto, server::{ModbusFrame, context::ModbusContext}};
/// use fixedvec::{FixedVec, alloc_stack}; // for std use regular std::vec::Vec
///
/// let mut ctx = ModbusContext::new();
///
/// let unit_id = 1;
/// loop {
///     let framebuf:ModbusFrameBuf = [0;256];
///     // read frame into the buffer
///     let mut mem = alloc_stack!([u8; 256]);
///     let mut response = FixedVec::new(&mut mem);
///     // create new frame processor object
///     let mut frame = ModbusFrame::new(unit_id, &framebuf, ModbusProto::TcpUdp, &mut response);
///     // parse frame buffer
///     if frame.parse().is_ok() {
///         // parsed ok
///         if frame.processing_required {
///             // call function depending is the request read-only or not
///             // a little more typing, but allows to lock context only for reading when writing
///             // isn't required
///             let result = match frame.readonly {
///                 true => frame.process_read(&ctx),
///                 false => frame.process_write(&mut ctx)
///             };
///             if result.is_err() {
///                 // fn error is returned at this point only if there's no space in response vec
///                 continue;
///             }
///         }
///         // processing is over (if required), let's check is sending the response required
///         if frame.response_required {
///             // sets Modbus error if happened, for RTU/ASCII frames adds CRC/LRU
///             frame.finalize_response();
///             response.as_slice(); // send response somewhere
///         }
///     }
/// }
/// ```
pub struct ModbusFrame<'a, V: VectorTrait<u8>> {
    pub unit_id: u8,
    buf: &'a ModbusFrameBuf,
    pub response: &'a mut V,
    pub proto: ModbusProto,
    /// after parse: is processing required
    pub processing_required: bool,
    /// is response required
    pub response_required: bool,
    /// is request read-only
    pub readonly: bool,
    /// Modbus frame start in buf (0 for RTU/ASCII, 6 for TCP)
    pub frame_start: usize,
    /// function requested
    pub func: u8,
    /// starting register
    pub reg: u16,
    /// registers to process
    pub count: u16,
    /// error code
    pub error: u8,
}

impl<'a, V: VectorTrait<u8>> ModbusFrame<'a, V> {
    pub fn new(
        unit_id: u8,
        buf: &'a ModbusFrameBuf,
        proto: ModbusProto,
        response: &'a mut V,
    ) -> Self {
        response.clear_all();
        Self {
            unit_id: unit_id,
            buf: buf,
            func: 0,
            proto: proto,
            response: response,
            processing_required: false,
            readonly: true,
            response_required: false,
            frame_start: 0,
            count: 1,
            reg: 0,
            error: 0,
        }
    }
    /// Should be always called if response needs to be sent
    pub fn finalize_response(&mut self) -> Result<(), ErrorKind> {
        if self.error > 0 {
            match self.proto {
                ModbusProto::TcpUdp => {
                    if self
                        .response
                        .add_bulk(&[0, 3, self.unit_id, self.func + 0x80, self.error])
                        .is_err()
                    {
                        return Err(ErrorKind::OOB);
                    }
                }
                ModbusProto::Rtu | ModbusProto::Ascii => {
                    if self
                        .response
                        .add_bulk(&[self.unit_id, self.func + 0x80, self.error])
                        .is_err()
                    {
                        return Err(ErrorKind::OOB);
                    }
                }
            }
        }
        match self.proto {
            ModbusProto::Rtu => {
                let crc = calc_crc16(&self.response.get_slice(), self.response.get_len() as u8);
                match self.response.add_bulk(&crc.to_le_bytes()) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(ErrorKind::OOB),
                }
            }
            ModbusProto::Ascii => {
                let lrc = calc_lrc(&self.response.get_slice(), self.response.get_len() as u8);
                match self.response.add(lrc) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(ErrorKind::OOB),
                }
            }
            _ => Ok(()),
        }
    }
    /// Process write functions
    pub fn process_write(&mut self, ctx: &mut context::ModbusContext) -> Result<(), ErrorKind> {
        if self.func == MODBUS_SET_COIL {
            // func 5
            // write single coil
            let val = match u16::from_be_bytes([
                self.buf[self.frame_start + 4],
                self.buf[self.frame_start + 5],
            ]) {
                0xff00 => true,
                0x0000 => false,
                _ => {
                    self.error = MODBUS_ERROR_ILLEGAL_DATA_VALUE;
                    return Ok(());
                }
            };
            let result = ctx.set_coil(self.reg, val);
            if result.is_err() {
                self.error = MODBUS_ERROR_ILLEGAL_DATA_ADDRESS;
                return Ok(());
            } else {
                tcp_response_set_data_len!(self, 6);
                // 6b unit, func, reg, val
                if self
                    .response
                    .add_bulk(&self.buf[self.frame_start..self.frame_start + 6])
                    .is_err()
                {
                    return Err(ErrorKind::OOB);
                }
                return Ok(());
            }
        } else if self.func == MODBUS_SET_HOLDING {
            // func 6
            // write single register
            let val = u16::from_be_bytes([
                self.buf[self.frame_start + 4],
                self.buf[self.frame_start + 5],
            ]);
            let result = ctx.set_holding(self.reg, val);
            if result.is_err() {
                self.error = MODBUS_ERROR_ILLEGAL_DATA_ADDRESS;
                return Ok(());
            } else {
                tcp_response_set_data_len!(self, 6);
                // 6b unit, func, reg, val
                if self
                    .response
                    .add_bulk(&self.buf[self.frame_start..self.frame_start + 6])
                    .is_err()
                {
                    return Err(ErrorKind::OOB);
                }
                return Ok(());
            }
        } else if self.func == MODBUS_SET_COILS_BULK || self.func == MODBUS_SET_HOLDINGS_BULK {
            // funcs 15 & 16
            // write multiple coils / registers
            let bytes = self.buf[self.frame_start + 6];
            let result = match self.func {
                MODBUS_SET_COILS_BULK => ctx.set_coils_from_u8(
                    self.reg,
                    self.count,
                    &self.buf[self.frame_start + 7..self.frame_start + 7 + bytes as usize],
                ),
                MODBUS_SET_HOLDINGS_BULK => ctx.set_holdings_from_u8(
                    self.reg,
                    &self.buf[self.frame_start + 7..self.frame_start + 7 + bytes as usize],
                ),
                _ => panic!(), // never reaches
            };
            match result {
                Ok(_) => {
                    tcp_response_set_data_len!(self, 6);
                    // 6b unit, f, reg, cnt
                    if self
                        .response
                        .add_bulk(&self.buf[self.frame_start..self.frame_start + 6])
                        .is_err()
                    {
                        return Err(ErrorKind::OOB);
                    }
                    return Ok(());
                }
                Err(_) => {
                    self.error = MODBUS_ERROR_ILLEGAL_DATA_ADDRESS;
                    return Ok(());
                }
            }
        }
        Ok(())
    }
    /// Process read functions
    pub fn process_read(&mut self, ctx: &context::ModbusContext) -> Result<(), ErrorKind> {
        if self.func == MODBUS_GET_COILS || self.func == MODBUS_GET_DISCRETES {
            // funcs 1 - 2
            // read coils / discretes
            let mut data_len = self.count >> 3;
            if self.count % 8 != 0 {
                data_len = data_len + 1;
            }
            tcp_response_set_data_len!(self, data_len + 3);
            if self
                .response
                .add_bulk(&self.buf[self.frame_start..self.frame_start + 2]) // 2b unit and func
                .is_err()
            {
                return Err(ErrorKind::OOB);
            }
            if self.response.add(data_len as u8).is_err() {
                // 1b data len
                return Err(ErrorKind::OOB);
            }
            let result = match self.func {
                MODBUS_GET_COILS => ctx.get_coils_as_u8(self.reg, self.count, self.response),
                MODBUS_GET_DISCRETES => {
                    ctx.get_discretes_as_u8(self.reg, self.count, self.response)
                }
                _ => panic!(), // never reaches
            };
            return match result {
                Ok(_) => Ok(()),
                Err(ErrorKind::OOBContext) => {
                    self.response.cut_end(5, 0);
                    self.error = MODBUS_ERROR_ILLEGAL_DATA_ADDRESS;
                    Ok(())
                }
                Err(_) => Err(ErrorKind::OOB),
            };
        } else if self.func == MODBUS_GET_HOLDINGS || self.func == MODBUS_GET_INPUTS {
            // funcs 3 - 4
            // read holdings / inputs
            let data_len = self.count << 1;
            tcp_response_set_data_len!(self, data_len + 3);
            if self
                .response
                .add_bulk(&self.buf[self.frame_start..self.frame_start + 2]) // 2b unit and func
                .is_err()
            {
                return Err(ErrorKind::OOB);
            }
            if self.response.add(data_len as u8).is_err() {
                // 1b data len
                return Err(ErrorKind::OOB);
            }
            let result = match self.func {
                MODBUS_GET_HOLDINGS => ctx.get_holdings_as_u8(self.reg, self.count, self.response),
                MODBUS_GET_INPUTS => ctx.get_inputs_as_u8(self.reg, self.count, self.response),
                _ => panic!(), // never reaches
            };
            return match result {
                Ok(_) => Ok(()),
                Err(ErrorKind::OOBContext) => {
                    self.response.cut_end(5, 0);
                    self.error = MODBUS_ERROR_ILLEGAL_DATA_ADDRESS;
                    Ok(())
                }
                Err(_) => Err(ErrorKind::OOB),
            };
        }
        Ok(())
    }
    /// Parse frame buffer
    pub fn parse(&mut self) -> Result<(), ErrorKind> {
        if self.proto == ModbusProto::TcpUdp {
            //let tr_id = u16::from_be_bytes([self.buf[0], self.buf[1]]);
            let proto_id = u16::from_be_bytes([self.buf[2], self.buf[3]]);
            let length = u16::from_be_bytes([self.buf[4], self.buf[5]]);
            if proto_id != 0 || length < 6 || length > 250 {
                return Err(ErrorKind::FrameBroken);
            }
            self.frame_start = 6;
        }
        let unit = self.buf[self.frame_start];
        let broadcast = unit == 0 || unit == 255; // some clients send broadcast to 0xff
        if !broadcast && unit != self.unit_id {
            return Ok(());
        }
        if !broadcast && self.proto == ModbusProto::TcpUdp {
            // copy 4 bytes: tr id and proto
            if self.response.add_bulk(&self.buf[0..4]).is_err() {
                return Err(ErrorKind::OOB);
            }
        }
        self.func = self.buf[self.frame_start + 1];
        macro_rules! check_frame_crc {
            ($len:expr) => {
                self.proto == ModbusProto::TcpUdp
                    || (self.proto == ModbusProto::Rtu
                        && calc_crc16(self.buf, $len)
                            == u16::from_le_bytes([
                                self.buf[$len as usize],
                                self.buf[$len as usize + 1],
                            ]))
                    || (self.proto == ModbusProto::Ascii
                        && calc_lrc(self.buf, $len) == self.buf[$len as usize])
            };
        }
        if self.func == MODBUS_GET_COILS || self.func == MODBUS_GET_DISCRETES {
            // funcs 1 - 2
            // read coils / discretes
            if broadcast {
                return Ok(());
            }
            if !check_frame_crc!(6) {
                return Err(ErrorKind::FrameCRCError);
            }
            self.response_required = true;
            self.count = u16::from_be_bytes([
                self.buf[self.frame_start + 4],
                self.buf[self.frame_start + 5],
            ]);
            if self.count > 2000 {
                self.error = MODBUS_ERROR_ILLEGAL_DATA_VALUE;
                return Ok(());
            }
            self.processing_required = true;
            self.reg = u16::from_be_bytes([
                self.buf[self.frame_start + 2],
                self.buf[self.frame_start + 3],
            ]);
            return Ok(());
        } else if self.func == MODBUS_GET_HOLDINGS || self.func == MODBUS_GET_INPUTS {
            // funcs 3 - 4
            // read holdings / inputs
            if broadcast {
                return Ok(());
            }
            if !check_frame_crc!(6) {
                return Err(ErrorKind::FrameCRCError);
            }
            self.response_required = true;
            self.count = u16::from_be_bytes([
                self.buf[self.frame_start + 4],
                self.buf[self.frame_start + 5],
            ]);
            if self.count > 125 {
                self.error = MODBUS_ERROR_ILLEGAL_DATA_VALUE;
                return Ok(());
            }
            self.processing_required = true;
            self.reg = u16::from_be_bytes([
                self.buf[self.frame_start + 2],
                self.buf[self.frame_start + 3],
            ]);
            return Ok(());
        } else if self.func == MODBUS_SET_COIL {
            // func 5
            // write single coil
            if !check_frame_crc!(6) {
                return Err(ErrorKind::FrameCRCError);
            }
            if !broadcast {
                self.response_required = true;
            }
            self.processing_required = true;
            self.readonly = false;
            self.reg = u16::from_be_bytes([
                self.buf[self.frame_start + 2],
                self.buf[self.frame_start + 3],
            ]);
            return Ok(());
        } else if self.func == MODBUS_SET_HOLDING {
            // func 6
            // write single register
            if !check_frame_crc!(6) {
                return Err(ErrorKind::FrameCRCError);
            }
            if !broadcast {
                self.response_required = true;
            }
            self.processing_required = true;
            self.readonly = false;
            self.reg = u16::from_be_bytes([
                self.buf[self.frame_start + 2],
                self.buf[self.frame_start + 3],
            ]);
            return Ok(());
        } else if self.func == MODBUS_SET_COILS_BULK || self.func == MODBUS_SET_HOLDINGS_BULK {
            // funcs 15 & 16
            // write multiple coils / registers
            let bytes = self.buf[self.frame_start + 6];
            if !check_frame_crc!(7 + bytes) {
                return Err(ErrorKind::FrameCRCError);
            }
            if !broadcast {
                self.response_required = true;
            }
            if bytes > 242 {
                self.error = MODBUS_ERROR_ILLEGAL_DATA_VALUE;
                return Ok(());
            }
            self.processing_required = true;
            self.readonly = false;
            self.reg = u16::from_be_bytes([
                self.buf[self.frame_start + 2],
                self.buf[self.frame_start + 3],
            ]);
            self.count = u16::from_be_bytes([
                self.buf[self.frame_start + 4],
                self.buf[self.frame_start + 5],
            ]);
            return Ok(());
        } else {
            // function unsupported
            if !broadcast {
                self.response_required = true;
                self.error = MODBUS_ERROR_ILLEGAL_FUNCTION;
            }
            return Ok(());
        }
    }
}
