pub mod context;

use crate::consts::{
    MODBUS_ERROR_ILLEGAL_DATA_ADDRESS, MODBUS_ERROR_ILLEGAL_DATA_VALUE,
    MODBUS_ERROR_ILLEGAL_FUNCTION, MODBUS_GET_COILS, MODBUS_GET_DISCRETES, MODBUS_GET_HOLDINGS,
    MODBUS_GET_INPUTS, MODBUS_SET_COIL, MODBUS_SET_COILS_BULK, MODBUS_SET_HOLDING,
    MODBUS_SET_HOLDINGS_BULK,
};
use crate::{calc_crc16, calc_lrc, ErrorKind, ModbusFrameBuf, ModbusProto, VectorTrait};

/// Modbus frame processor
///
/// ```no_run
/// # #[cfg(feature = "fixedvec")]
/// # mod with_fixedvec {
/// use rmodbus::{ModbusFrameBuf, ModbusProto, server::{ModbusFrame, context::ModbusContextFull}};
/// use fixedvec::{FixedVec, alloc_stack}; // for std use regular std::vec::Vec
///
/// # fn code() {
/// let mut ctx = ModbusContextFull::new();
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
///             // call a function depending is the request read-only or not
///             // a little more typing, but allows to lock the context only for reading if writing
///             // isn't required
///             let result = match frame.readonly {
///                 true => frame.process_read(&ctx),
///                 false => frame.process_write(&mut ctx)
///             };
///             if result.is_err() {
///                 // fn error is returned at this point only if there's no space in the response
///                 // vec (so can be caused in nostd only)
///                 continue;
///             }
///         }
///         // processing is over (if required), let's check is the response required
///         if frame.response_required {
///             // sets Modbus error if happened, for RTU/ASCII frames adds CRC/LRU
///             frame.finalize_response();
///             response.as_slice(); // send response somewhere
///         }
///     }
/// }
/// # } }
/// ```

macro_rules! tcp_response_set_data_len {
    ($self: expr, $len:expr) => {
        if $self.proto == ModbusProto::TcpUdp {
            $self.response.extend(&($len as u16).to_be_bytes())?;
        }
    };
}

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
        response.clear();
        Self {
            unit_id,
            buf,
            func: 0,
            proto,
            response,
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
                    self.response
                        .extend(&[0, 3, self.unit_id, self.func + 0x80, self.error])?;
                }
                ModbusProto::Rtu | ModbusProto::Ascii => {
                    self.response
                        .extend(&[self.unit_id, self.func + 0x80, self.error])?;
                }
            }
        }
        match self.proto {
            ModbusProto::Rtu => {
                let len = self.response.len();
                if len > u8::MAX as usize {
                    return Err(ErrorKind::OOB);
                }
                #[allow(clippy::cast_possible_truncation)]
                let crc = calc_crc16(self.response.as_slice(), len as u8);
                self.response.extend(&crc.to_le_bytes())
            }
            ModbusProto::Ascii => {
                let len = self.response.len();
                if len > u8::MAX as usize {
                    return Err(ErrorKind::OOB);
                }
                #[allow(clippy::cast_possible_truncation)]
                let lrc = calc_lrc(self.response.as_slice(), len as u8);
                self.response.push(lrc)
            }
            ModbusProto::TcpUdp => Ok(()),
        }
    }
    /// Process write functions
    pub fn process_write<const C: usize, const D: usize, const I: usize, const H: usize>(
        &mut self,
        ctx: &mut context::ModbusContext<C, D, I, H>,
    ) -> Result<(), ErrorKind> {
        match self.func {
            MODBUS_SET_COIL => {
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
                if ctx.set_coil(self.reg, val).is_err() {
                    self.error = MODBUS_ERROR_ILLEGAL_DATA_ADDRESS;
                    return Ok(());
                }
                tcp_response_set_data_len!(self, 6);
                // 6b unit, func, reg, val
                self.response
                    .extend(&self.buf[self.frame_start..self.frame_start + 6])
            }
            MODBUS_SET_HOLDING => {
                // func 6
                // write single register
                let val = u16::from_be_bytes([
                    self.buf[self.frame_start + 4],
                    self.buf[self.frame_start + 5],
                ]);
                if ctx.set_holding(self.reg, val).is_err() {
                    self.error = MODBUS_ERROR_ILLEGAL_DATA_ADDRESS;
                    return Ok(());
                }
                tcp_response_set_data_len!(self, 6);
                // 6b unit, func, reg, val
                self.response
                    .extend(&self.buf[self.frame_start..self.frame_start + 6])
            }
            MODBUS_SET_COILS_BULK | MODBUS_SET_HOLDINGS_BULK => {
                // funcs 15 & 16
                // write multiple coils / registers
                let bytes = self.buf[self.frame_start + 6];
                let result = if self.func == MODBUS_SET_COILS_BULK {
                    ctx.set_coils_from_u8(
                        self.reg,
                        self.count,
                        &self.buf[self.frame_start + 7..self.frame_start + 7 + bytes as usize],
                    )
                } else {
                    ctx.set_holdings_from_u8(
                        self.reg,
                        &self.buf[self.frame_start + 7..self.frame_start + 7 + bytes as usize],
                    )
                };
                if result.is_ok() {
                    tcp_response_set_data_len!(self, 6);
                    // 6b unit, f, reg, cnt
                    self.response
                        .extend(&self.buf[self.frame_start..self.frame_start + 6])
                } else {
                    self.error = MODBUS_ERROR_ILLEGAL_DATA_ADDRESS;
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
    /// Process read functions
    pub fn process_read<const C: usize, const D: usize, const I: usize, const H: usize>(
        &mut self,
        ctx: &context::ModbusContext<C, D, I, H>,
    ) -> Result<(), ErrorKind> {
        match self.func {
            MODBUS_GET_COILS | MODBUS_GET_DISCRETES => {
                // funcs 1 - 2
                // read coils / discretes
                let mut data_len = self.count >> 3;
                if self.count % 8 != 0 {
                    data_len += 1;
                }
                tcp_response_set_data_len!(self, data_len + 3);
                // 2b unit and func
                self.response
                    .extend(&self.buf[self.frame_start..self.frame_start + 2])?;
                if data_len > u16::from(u8::MAX) {
                    return Err(ErrorKind::OOB);
                }
                #[allow(clippy::cast_possible_truncation)]
                self.response.push(data_len as u8)?;
                let result = if self.func == MODBUS_GET_COILS {
                    ctx.get_coils_as_u8(self.reg, self.count, self.response)
                } else {
                    ctx.get_discretes_as_u8(self.reg, self.count, self.response)
                };
                if let Err(e) = result {
                    if e == ErrorKind::OOBContext {
                        self.response.cut_end(5, 0);
                        self.error = MODBUS_ERROR_ILLEGAL_DATA_ADDRESS;
                        Ok(())
                    } else {
                        Err(e)
                    }
                } else {
                    Ok(())
                }
            }
            MODBUS_GET_HOLDINGS | MODBUS_GET_INPUTS => {
                // funcs 3 - 4
                // read holdings / inputs
                let data_len = self.count << 1;
                tcp_response_set_data_len!(self, data_len + 3);
                // 2b unit and func
                self.response
                    .extend(&self.buf[self.frame_start..self.frame_start + 2])?;
                if data_len > u16::from(u8::MAX) {
                    return Err(ErrorKind::OOB);
                }
                #[allow(clippy::cast_possible_truncation)]
                // 1b data len
                self.response.push(data_len as u8)?;
                let result = if self.func == MODBUS_GET_HOLDINGS {
                    ctx.get_holdings_as_u8(self.reg, self.count, self.response)
                } else {
                    ctx.get_inputs_as_u8(self.reg, self.count, self.response)
                };
                if let Err(e) = result {
                    if e == ErrorKind::OOBContext {
                        self.response.cut_end(5, 0);
                        self.error = MODBUS_ERROR_ILLEGAL_DATA_ADDRESS;
                        Ok(())
                    } else {
                        Err(e)
                    }
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
    /// Parse frame buffer
    #[allow(clippy::too_many_lines)]
    pub fn parse(&mut self) -> Result<(), ErrorKind> {
        if self.proto == ModbusProto::TcpUdp {
            //let tr_id = u16::from_be_bytes([self.buf[0], self.buf[1]]);
            let proto_id = u16::from_be_bytes([self.buf[2], self.buf[3]]);
            let length = u16::from_be_bytes([self.buf[4], self.buf[5]]);
            if proto_id != 0 || !(6..=250).contains(&length) {
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
            self.response.extend(&self.buf[0..4])?;
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
        match self.func {
            MODBUS_GET_COILS | MODBUS_GET_DISCRETES => {
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
                Ok(())
            }
            MODBUS_GET_HOLDINGS | MODBUS_GET_INPUTS => {
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
                Ok(())
            }
            MODBUS_SET_COIL | MODBUS_SET_HOLDING => {
                // func 5 / 6
                // write single coil / register
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
                Ok(())
            }
            MODBUS_SET_COILS_BULK | MODBUS_SET_HOLDINGS_BULK => {
                // funcs 15 & 16
                // write multiple coils / registers
                let bytes = self.buf[self.frame_start + 6];
                if !check_frame_crc!(7 + bytes) {
                    return Err(ErrorKind::FrameCRCError);
                }
                if !broadcast {
                    self.response_required = true;
                }
                if bytes > 246 {
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
                Ok(())
            }
            _ => {
                // function unsupported
                if !broadcast {
                    self.response_required = true;
                    self.error = MODBUS_ERROR_ILLEGAL_FUNCTION;
                }
                Ok(())
            }
        }
    }

    /// Retrieve which fields of a [`ModbusContext`](`context::ModbusContext`) will be changed by applying this frame
    ///
    /// Returns None if no fields will be changed.
    pub fn changes(&self) -> Option<Changes> {
        let reg = self.reg;
        let count = self.count;

        Some(match self.func {
            MODBUS_SET_COIL => Changes::Coils { reg, count: 1 },
            MODBUS_SET_COILS_BULK => Changes::Coils { reg, count },
            MODBUS_SET_HOLDING => Changes::Holdings { reg, count: 1 },
            MODBUS_SET_HOLDINGS_BULK => Changes::Holdings { reg, count },
            _ => return None,
        })
    }
}

/// See [`ModbusFrame::changes`]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Changes {
    Coils { reg: u16, count: u16 },
    Holdings { reg: u16, count: u16 },
}
