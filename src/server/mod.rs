pub mod context;
pub mod representable;
pub mod storage;

use core::slice;
pub use representable::representations;

#[allow(clippy::wildcard_imports)]
use crate::consts::*;
use crate::{calc_crc16, calc_lrc, ErrorKind, ModbusProto, VectorTrait};

/// Modbus frame processor
///
/// ```no_run
/// # #[cfg(feature = "fixedvec")]
/// # mod with_fixedvec {
/// use rmodbus::{ModbusFrameBuf, ModbusProto, server::{ModbusFrame, storage::ModbusStorageFull, context::ModbusContext}};
/// use fixedvec::{FixedVec, alloc_stack}; // for std use regular std::vec::Vec
///
/// # fn code() {
/// let mut ctx = ModbusStorageFull::new();
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

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModbusFrame<'a, V: VectorTrait<u8>> {
    pub unit_id: u8,
    buf: &'a [u8],
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
    pub fn new(unit_id: u8, buf: &'a [u8], proto: ModbusProto, response: &'a mut V) -> Self {
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
                        // write 2b length 1b unit ID, 1b function code and 1b error
                        // 2b transaction ID and 2b protocol ID were already written by .parse()
                        .extend(&[0, 3, self.unit_id, self.func + 0x80, self.error])?;
                }
                ModbusProto::Rtu | ModbusProto::Ascii => {
                    self.response
                        // write 1b unit ID, 1b function code and 1b error
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
    pub fn process_write<C: context::ModbusContext>(
        &mut self,
        ctx: &mut C,
    ) -> Result<(), ErrorKind> {
        match self.func {
            MODBUS_SET_COIL => {
                // func 5
                // write single coil
                if self.buf.len() < self.frame_start + 6 {
                    return Err(ErrorKind::FrameBroken);
                }
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
                if self.buf.len() < self.frame_start + 6 {
                    return Err(ErrorKind::FrameBroken);
                }
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
                if self.buf.len() < self.frame_start + 7 {
                    return Err(ErrorKind::FrameBroken);
                }
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
            MODBUS_GET_HOLDINGS | MODBUS_GET_INPUTS | MODBUS_GET_COILS | MODBUS_GET_DISCRETES => {
                Err(ErrorKind::ReadCallOnWriteFrame)
            }
            _ => Ok(()),
        }
    }
    /// Construct [`Write`] struct describing the requested write.
    ///
    /// If you use this to process the requested write yourself (so not calling
    /// [`process_write`](ModbusFrame::process_write) with a
    /// [`ModbusContext`](context::ModbusContext)) don't forget to call
    /// [`process_external_write`](ModbusFrame::process_external_write), these two calls together
    /// replace the call to [`process_write`](ModbusFrame::process_write).
    pub fn get_external_write(&mut self) -> Result<Write, ErrorKind> {
        match self.func {
            MODBUS_SET_COIL => {
                // func 5
                // write single coil
                let val = match u16::from_be_bytes([
                    self.buf[self.frame_start + 4],
                    self.buf[self.frame_start + 5],
                ]) {
                    0xff00 => Write::Bits(WriteBits {
                        address: self.reg,
                        count: 1,
                        data: slice::from_ref(&1u8),
                    }),
                    0x0000 => Write::Bits(WriteBits {
                        address: self.reg,
                        count: 1,
                        data: slice::from_ref(&0u8),
                    }),
                    _ => {
                        self.set_modbus_error_if_unset(&ErrorKind::IllegalDataValue)?;
                        return Err(ErrorKind::IllegalDataValue);
                    }
                };

                Ok(val)
            }
            MODBUS_SET_HOLDING => {
                // func 6
                // write single register

                let write = Write::Words(WriteWords {
                    address: self.reg,
                    count: 1,
                    data: &self.buf[self.frame_start + 4..self.frame_start + 6],
                });

                Ok(write)
            }
            MODBUS_SET_COILS_BULK => {
                // funcs 15 & 16
                // write multiple coils / registers
                let bytes = self.buf[self.frame_start + 6];
                let data_start = self.frame_start + 7;

                let write = Write::Bits(WriteBits {
                    address: self.reg,
                    count: self.count,
                    data: &self.buf[data_start..data_start + bytes as usize],
                });

                Ok(write)
            }
            MODBUS_SET_HOLDINGS_BULK => {
                // funcs 15 & 16
                // write multiple coils / registers
                let bytes = self.buf[self.frame_start + 6];
                let data_start = self.frame_start + 7;

                let write = Write::Words(WriteWords {
                    address: self.reg,
                    count: self.count,
                    data: &self.buf[data_start..data_start + bytes as usize],
                });

                Ok(write)
            }
            MODBUS_GET_HOLDINGS | MODBUS_GET_INPUTS | MODBUS_GET_COILS | MODBUS_GET_DISCRETES => {
                Err(ErrorKind::ReadCallOnWriteFrame)
            }
            _ => {
                self.set_modbus_error_if_unset(&ErrorKind::IllegalFunction)?;
                Err(ErrorKind::IllegalFunction)
            }
        }
    }
    /// See [get_external_write](ModbusFrame::get_external_write)
    pub fn process_external_write(
        &mut self,
        write_result: Result<(), ErrorKind>,
    ) -> Result<(), ErrorKind> {
        match write_result {
            Ok(()) => {
                match self.func {
                    MODBUS_SET_COIL
                    | MODBUS_SET_HOLDING
                    | MODBUS_SET_COILS_BULK
                    | MODBUS_SET_HOLDINGS_BULK => {
                        // funcs 5 & 6
                        // write single coil / register
                        // funcs 15 & 16
                        // write multiple coils / registers

                        tcp_response_set_data_len!(self, 6);
                        // 6b unit, func, reg, val
                        self.response
                            .extend(&self.buf[self.frame_start..self.frame_start + 6])
                    }
                    MODBUS_GET_HOLDINGS | MODBUS_GET_INPUTS | MODBUS_GET_COILS
                    | MODBUS_GET_DISCRETES => Err(ErrorKind::ReadCallOnWriteFrame),
                    _ => {
                        self.set_modbus_error_if_unset(&ErrorKind::IllegalFunction)?;
                        Err(ErrorKind::IllegalFunction)
                    }
                }
            }
            Err(e) if e.is_modbus_error() => {
                self.set_modbus_error_if_unset(&e)?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Process read functions
    pub fn process_read<C: context::ModbusContext>(&mut self, ctx: &C) -> Result<(), ErrorKind> {
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
            MODBUS_SET_COIL
            | MODBUS_SET_HOLDING
            | MODBUS_SET_COILS_BULK
            | MODBUS_SET_HOLDINGS_BULK => Err(ErrorKind::WriteCallOnReadFrame),
            _ => Ok(()),
        }
    }

    /// Construct [`Read`] struct describing the requested read.
    ///
    /// If you use this to process the requested read yourself (so not calling
    /// [`process_read`](ModbusFrame::process_read) with a
    /// [`ModbusContext`](context::ModbusContext)) don't forget to call
    /// [`process_external_read`](ModbusFrame::process_external_read), these two calls together
    /// replace the call to [`process_read`](ModbusFrame::process_read).
    pub fn get_external_read(&mut self) -> Result<Read, ErrorKind> {
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
                // 1b data len
                self.response.push(data_len as u8)?;

                // extend with data_len so we can get the extra space as &mut slice for Read struct
                let current_length = self.response.len();
                let new_length = current_length + data_len as usize;
                self.response.resize(new_length, 0u8)?;

                Ok(Read::Bits(ReadBits {
                    address: self.reg,
                    count: self.count,
                    buf: &mut self.response.as_mut_slice()[current_length..new_length],
                }))
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

                // extend with data_len so we can get the extra space as &mut slice for Read struct
                let current_length = self.response.len();
                let new_length = current_length + data_len as usize;
                self.response.resize(new_length, 0u8)?;

                Ok(Read::Words(ReadWords {
                    address: self.reg,
                    count: self.count,
                    buf: &mut self.response.as_mut_slice()[current_length..new_length],
                }))
            }
            MODBUS_SET_COIL
            | MODBUS_SET_HOLDING
            | MODBUS_SET_COILS_BULK
            | MODBUS_SET_HOLDINGS_BULK => Err(ErrorKind::WriteCallOnReadFrame),
            _ => {
                self.set_modbus_error_if_unset(&ErrorKind::IllegalFunction)?;
                Err(ErrorKind::IllegalFunction)
            }
        }
    }

    /// see [get_external_read](ModbusFrame::get_external_read)
    pub fn process_external_read(
        &mut self,
        read_result: Result<(), ErrorKind>,
    ) -> Result<(), ErrorKind> {
        match read_result {
            Ok(()) => match self.func {
                MODBUS_GET_COILS | MODBUS_GET_DISCRETES | MODBUS_GET_HOLDINGS
                | MODBUS_GET_INPUTS => Ok(()),
                MODBUS_SET_COIL
                | MODBUS_SET_HOLDING
                | MODBUS_SET_COILS_BULK
                | MODBUS_SET_HOLDINGS_BULK => Err(ErrorKind::WriteCallOnReadFrame),
                _ => {
                    self.set_modbus_error_if_unset(&ErrorKind::IllegalFunction)?;
                    Ok(())
                }
            },
            Err(e) if e.is_modbus_error() => {
                self.set_modbus_error_if_unset(&e)?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Parse frame buffer
    #[allow(clippy::too_many_lines)]
    pub fn parse(&mut self) -> Result<(), ErrorKind> {
        if self.proto == ModbusProto::TcpUdp {
            if self.buf.len() < 6 {
                return Err(ErrorKind::FrameBroken);
            }
            //let tr_id = u16::from_be_bytes([self.buf[0], self.buf[1]]);
            let proto_id = u16::from_be_bytes([self.buf[2], self.buf[3]]);
            let length = u16::from_be_bytes([self.buf[4], self.buf[5]]);
            if proto_id != 0 || !(6..=250).contains(&length) {
                return Err(ErrorKind::FrameBroken);
            }
            self.frame_start = 6;
        }
        if self.frame_start >= self.buf.len() {
            return Err(ErrorKind::FrameBroken);
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
        if self.buf.len() < self.frame_start + 2 {
            return Err(ErrorKind::FrameBroken);
        }
        self.func = self.buf[self.frame_start + 1];
        macro_rules! check_frame_crc {
            ($len:expr) => {
                match self.proto {
                    ModbusProto::TcpUdp => true,
                    ModbusProto::Rtu => {
                        if self.buf.len() < self.frame_start + $len as usize + 2 {
                            return Err(ErrorKind::FrameBroken);
                        }
                        calc_crc16(self.buf, $len)
                            == u16::from_le_bytes([
                                self.buf[self.frame_start + $len as usize],
                                self.buf[self.frame_start + $len as usize + 1],
                            ])
                    }
                    ModbusProto::Ascii => {
                        if self.buf.len() < self.frame_start + $len as usize + 1 {
                            return Err(ErrorKind::FrameBroken);
                        }
                        calc_lrc(self.buf, $len) == self.buf[self.frame_start + $len as usize]
                    }
                }
            };
        }
        match self.func {
            MODBUS_GET_COILS | MODBUS_GET_DISCRETES => {
                // funcs 1 - 2
                // read coils / discretes
                if broadcast {
                    return Ok(());
                }
                if self.buf.len() < self.frame_start + 6 {
                    return Err(ErrorKind::FrameBroken);
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
                if self.buf.len() < self.frame_start + 6 {
                    return Err(ErrorKind::FrameBroken);
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
                if self.buf.len() < self.frame_start + 4 {
                    return Err(ErrorKind::FrameBroken);
                }
                if !check_frame_crc!(6) {
                    return Err(ErrorKind::FrameCRCError);
                }
                if !broadcast {
                    self.response_required = true;
                }
                self.count = 1;
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
                if self.buf.len() < self.frame_start + 7 {
                    return Err(ErrorKind::FrameBroken);
                }
                let bytes = self.buf[self.frame_start + 6];
                if !check_frame_crc!(7 + bytes) {
                    return Err(ErrorKind::FrameCRCError);
                }
                if !broadcast {
                    self.response_required = true;
                }
                self.count = u16::from_be_bytes([
                    self.buf[self.frame_start + 4],
                    self.buf[self.frame_start + 5],
                ]);
                let max_count = if self.func == MODBUS_SET_COILS_BULK {
                    1968
                } else {
                    123
                };
                if self.count > max_count {
                    self.error = MODBUS_ERROR_ILLEGAL_DATA_VALUE;
                    return Ok(());
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

    /// If the error field on the [`ModbusFrame`] isn't already set this function will set it and
    /// resize the response buffer to what's expected by [`ModbusFrame::finalize_response`]
    ///
    /// # Panics
    ///
    /// Should not panic
    pub fn set_modbus_error_if_unset(&mut self, err: &ErrorKind) -> Result<(), ErrorKind> {
        if self.error == 0 && err.is_modbus_error() {
            // leave 0 bytes for RTU/ASCII, leave 4 bytes for TCP/UDP (Transaction ID and Protocol ID)
            let len_leave_before_finalize = if self.proto == ModbusProto::TcpUdp {
                4
            } else {
                0
            };

            self.response.resize(len_leave_before_finalize, 0)?;
            self.error = err
                .to_modbus_error()
                .expect("the outer if statement checks for this");
        }
        Ok(())
    }
}

/// See [`ModbusFrame::changes`]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Changes {
    Coils { reg: u16, count: u16 },
    Holdings { reg: u16, count: u16 },
}

/// See [`get_external_write`](ModbusFrame::get_external_write)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WriteBits<'a> {
    pub address: u16,
    pub count: u16,
    pub data: &'a [u8],
}

/// See [`get_external_write`](ModbusFrame::get_external_write)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WriteWords<'a> {
    pub address: u16,
    pub count: u16,
    pub data: &'a [u8],
}

/// See [`get_external_write`](ModbusFrame::get_external_write)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Write<'a> {
    Bits(WriteBits<'a>),
    Words(WriteWords<'a>),
}

/// See [`get_external_read`](ModbusFrame::get_external_read)
#[derive(Debug, Eq, PartialEq)]
pub struct ReadBits<'a> {
    pub address: u16,
    pub count: u16,
    pub buf: &'a mut [u8],
}

/// See [`get_external_read`](ModbusFrame::get_external_read)
#[derive(Debug, Eq, PartialEq)]
pub struct ReadWords<'a> {
    pub address: u16,
    pub count: u16,
    pub buf: &'a mut [u8],
}

/// See [`get_external_read`](ModbusFrame::get_external_read)
#[derive(Debug, Eq, PartialEq)]
pub enum Read<'a> {
    Bits(ReadBits<'a>),
    Words(ReadWords<'a>),
}
