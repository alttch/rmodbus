use super::super::{ErrorKind, VectorTrait};
use ieee754::Ieee754;

// TODO variable length when const_generics become stable

#[cfg(not(feature = "smallcontext"))]
pub const CONTEXT_SIZE: usize = 10_000; // divisible by 8 w/o remainder

#[cfg(feature = "smallcontext")]
pub const CONTEXT_SIZE: usize = 1_000;

#[derive(Eq, PartialEq, Debug)]
pub enum ModbusContextRegister {
    Coil,
    Discrete,
    Input,
    Holding,
}

/// Contains standard Modbus register contexts
pub struct ModbusContext {
    pub coils: [bool; CONTEXT_SIZE],
    pub discretes: [bool; CONTEXT_SIZE],
    pub inputs: [u16; CONTEXT_SIZE],
    pub holdings: [u16; CONTEXT_SIZE],
}

macro_rules! get_regs_as_u8 {
    ($reg_context:expr, $reg:expr, $count:expr, $result:expr) => {
        let reg_to = $reg as usize + $count as usize;
        if reg_to > CONTEXT_SIZE {
            return Err(ErrorKind::OOBContext);
        }
        for c in $reg as usize..reg_to {
            if $result.add(($reg_context[c] >> 8) as u8).is_err() {
                return Err(ErrorKind::OOB);
            }
            if $result.add($reg_context[c] as u8).is_err() {
                return Err(ErrorKind::OOB);
            }
        }
        return Ok(());
    };
}

macro_rules! get_bools_as_u8 {
    ($reg_context:expr, $reg:expr, $count:expr, $result:expr) => {
        let reg_to = $reg as usize + $count as usize;
        if reg_to > CONTEXT_SIZE {
            return Err(ErrorKind::OOBContext);
        }
        let mut creg = $reg as usize;
        while creg < reg_to {
            let mut cbyte = 0;
            for i in 0..8 {
                if $reg_context[creg] {
                    cbyte = cbyte | 1 << i
                }
                creg += 1;
                if creg >= reg_to {
                    break;
                }
            }
            if $result.add(cbyte).is_err() {
                return Err(ErrorKind::OOB);
            };
        }
        return Ok(());
    };
}

macro_rules! set_regs_from_u8 {
    ($reg_context:expr, $reg:expr, $values:expr) => {
        let len = $values.len();
        if $reg as usize + len / 2 > CONTEXT_SIZE {
            return Err(ErrorKind::OOBContext);
        }
        let mut i = 0;
        let mut creg = $reg as usize;
        while i < len {
            $reg_context[creg] = u16::from_be_bytes([
                $values[i],
                match i + 1 < len {
                    true => $values[i + 1],
                    false => return Err(ErrorKind::OOB),
                },
            ]);
            i += 2;
            creg += 1;
        }
        return Ok(());
    };
}

macro_rules! set_bools_from_u8 {
    ($reg_context:expr, $reg:expr, $count:expr, $values:expr) => {
        let reg_to = $reg as usize + $count as usize;
        if reg_to > CONTEXT_SIZE {
            return Err(ErrorKind::OOBContext);
        }
        let mut creg = $reg as usize;
        let mut cbyte = 0;
        let mut cnt = 0;
        let len = $values.len();
        while creg < reg_to && cnt < $count {
            if cbyte >= len {
                return Err(ErrorKind::OOB);
            }
            let mut b: u8 = $values[cbyte];
            for _ in 0..8 {
                $reg_context[creg] = b & 1 == 1;
                b = b >> 1;
                creg = creg + 1;
                cnt = cnt + 1;
                if cnt == $count || creg == reg_to {
                    break;
                }
            }
            cbyte = cbyte + 1;
        }
        return Ok(());
    };
}

macro_rules! get_bulk {
    ($reg_context:expr, $reg:expr, $count:expr, $result:expr) => {
        let reg_to = $reg as usize + $count as usize;
        if reg_to > CONTEXT_SIZE {
            return Err(ErrorKind::OOBContext);
        }
        if $result
            .add_bulk(&$reg_context[$reg as usize..reg_to])
            .is_err()
        {
            return Err(ErrorKind::OOB);
        }
        return Ok(());
    };
}

macro_rules! set_bulk {
    ($reg_context:expr, $reg:expr, $values:expr) => {
        if $reg as usize + $values.len() > CONTEXT_SIZE {
            return Err(ErrorKind::OOBContext);
        }
        for (i, value) in $values.iter().enumerate() {
            $reg_context[$reg as usize + i] = *value;
        }
        return Ok(());
    };
}

macro_rules! get {
    ($reg_context:expr, $reg:expr) => {
        if $reg as usize >= CONTEXT_SIZE {
            return Err(ErrorKind::OOBContext);
        }
        return Ok($reg_context[$reg as usize]);
    };
}

macro_rules! set {
    ($reg_context:expr, $reg:expr, $value:expr) => {
        if $reg as usize >= CONTEXT_SIZE {
            return Err(ErrorKind::OOBContext);
        }
        $reg_context[$reg as usize] = $value;
        return Ok(());
    };
}

macro_rules! get_u32 {
    ($reg_context:expr, $reg:expr) => {
        if $reg as usize + 1 >= CONTEXT_SIZE {
            return Err(ErrorKind::OOBContext);
        }
        return Ok(
            (($reg_context[$reg as usize] as u32) << 16) + $reg_context[($reg as usize) + 1] as u32
        );
    };
}

macro_rules! set_u32 {
    ($reg_context:expr, $reg:expr, $value:expr) => {
        if $reg as usize + 1 >= CONTEXT_SIZE {
            return Err(ErrorKind::OOBContext);
        }
        $reg_context[$reg as usize] = ($value >> 16) as u16;
        $reg_context[$reg as usize + 1] = $value as u16;
        return Ok(());
    };
}

macro_rules! get_u64 {
    ($reg_context:expr, $reg:expr) => {
        if $reg as usize + 3 >= CONTEXT_SIZE {
            return Err(ErrorKind::OOBContext);
        }
        return Ok((($reg_context[$reg as usize] as u64) << 48)
            + (($reg_context[$reg as usize + 1] as u64) << 32)
            + (($reg_context[$reg as usize + 2] as u64) << 16)
            + $reg_context[($reg as usize) + 3] as u64);
    };
}

macro_rules! set_u64 {
    ($reg_context:expr, $reg:expr, $value:expr) => {
        if $reg as usize + 3 >= CONTEXT_SIZE {
            return Err(ErrorKind::OOBContext);
        }
        $reg_context[$reg as usize] = ($value >> 48) as u16;
        $reg_context[$reg as usize + 1] = ($value >> 32) as u16;
        $reg_context[$reg as usize + 2] = ($value >> 16) as u16;
        $reg_context[$reg as usize + 3] = $value as u16;
        return Ok(());
    };
}

impl ModbusContext {
    pub fn new() -> Self {
        return Self {
            coils: [false; CONTEXT_SIZE],
            discretes: [false; CONTEXT_SIZE],
            inputs: [0; CONTEXT_SIZE],
            holdings: [0; CONTEXT_SIZE],
        };
    }

    pub fn clear_all(&mut self) {
        for i in 0..CONTEXT_SIZE {
            self.coils[i] = false;
            self.discretes[i] = false;
            self.inputs[i] = 0;
            self.holdings[i] = 0;
        }
    }

    pub fn clear_coils(&mut self) {
        for i in 0..CONTEXT_SIZE {
            self.coils[i] = false;
        }
    }

    pub fn clear_discretes(&mut self) {
        for i in 0..CONTEXT_SIZE {
            self.discretes[i] = false;
        }
    }

    pub fn clear_inputs(&mut self) {
        for i in 0..CONTEXT_SIZE {
            self.inputs[i] = 0;
        }
    }

    pub fn clear_holdings(&mut self) {
        for i in 0..CONTEXT_SIZE {
            self.holdings[i] = 0;
        }
    }

    /// Create context iterator
    ///
    /// Iterate Modbus context as u8
    ///
    /// Useful for dump creation. To restore dump back, use "set_cell"
    /// or "create_writer"
    ///```
    ///use rmodbus::server::context::*;
    ///
    ///let mut ctx = ModbusContext::new();
    ///
    ///for value in ctx.iter() {
    ///    // store value somewhere
    ///}
    pub fn iter(&mut self) -> ModbusContextIterator {
        return ModbusContextIterator { curr: 0, ctx: self };
    }

    /// Iterate Modbus context as u8
    ///
    /// Useful for dump creation. To restore dump back, use "set_context_cell"
    /// or ModbusContextWriter::new()
    ///
    pub fn create_writer(&mut self) -> ModbusContextWriter {
        return ModbusContextWriter { curr: 0, ctx: self };
    }

    /// Get inputs as Vec<u8>
    ///
    /// Note: Vec is always appended
    pub fn get_inputs_as_u8<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_regs_as_u8!(self.inputs, reg, count, result);
    }

    /// Get holdings as Vec<u8>
    ///
    /// Note: Vec is always appended
    pub fn get_holdings_as_u8<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_regs_as_u8!(self.holdings, reg, count, result);
    }

    /// Set inputs from &Vec<u8>
    pub fn set_inputs_from_u8(&mut self, reg: u16, values: &[u8]) -> Result<(), ErrorKind> {
        set_regs_from_u8!(self.inputs, reg, values);
    }

    /// Set holdings from &Vec<u8>
    pub fn set_holdings_from_u8(&mut self, reg: u16, values: &[u8]) -> Result<(), ErrorKind> {
        set_regs_from_u8!(self.holdings, reg, values);
    }

    /// Get coils as Vec<u8>
    ///
    /// Note: Vec is always appended
    pub fn get_coils_as_u8<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_bools_as_u8!(self.coils, reg, count, result);
    }

    /// Get discretes as Vec<u8>
    ///
    /// Note: Vec is always appended
    pub fn get_discretes_as_u8<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_bools_as_u8!(self.discretes, reg, count, result);
    }

    /// Set coils from &Vec<u8>
    ///
    /// As coils are packed in u8, parameter *count* specifies how many coils are actually needed
    /// to set, extra bits are ignored
    pub fn set_coils_from_u8(
        &mut self,
        reg: u16,
        count: u16,
        values: &[u8],
    ) -> Result<(), ErrorKind> {
        set_bools_from_u8!(self.coils, reg, count, values);
    }

    /// Set discretes from &Vec<u8>
    ///
    /// As discretes are packed in u8, parameter *count* specifies how many coils are actually
    /// needed to set, extra bits are ignored
    pub fn set_discretes_from_u8(
        &mut self,
        reg: u16,
        count: u16,
        values: &[u8],
    ) -> Result<(), ErrorKind> {
        set_bools_from_u8!(self.discretes, reg, count, values);
    }

    /// Bulk get coils
    ///
    /// Note: Vec is always appended
    pub fn get_coils_bulk<V: VectorTrait<bool>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_bulk!(self.coils, reg, count, result);
    }

    /// Bulk get discretes
    ///
    /// Note: Vec is always appended
    pub fn get_discretes_bulk<V: VectorTrait<bool>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_bulk!(self.discretes, reg, count, result);
    }

    /// Bulk get inputs
    ///
    /// Note: Vec is always appended
    pub fn get_inputs_bulk<V: VectorTrait<u16>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_bulk!(self.inputs, reg, count, result);
    }

    /// Bulk get holdings
    ///
    /// Note: Vec is always appended
    pub fn get_holdings_bulk<V: VectorTrait<u16>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_bulk!(self.holdings, reg, count, result);
    }

    /// Bulk set coils
    pub fn set_coils_bulk(&mut self, reg: u16, values: &[bool]) -> Result<(), ErrorKind> {
        set_bulk!(self.coils, reg, values);
    }

    /// Bulk set discretes
    pub fn set_discretes_bulk(&mut self, reg: u16, values: &[bool]) -> Result<(), ErrorKind> {
        set_bulk!(self.discretes, reg, values);
    }

    /// Bulk set inputs
    pub fn set_inputs_bulk(&mut self, reg: u16, values: &[u16]) -> Result<(), ErrorKind> {
        set_bulk!(self.inputs, reg, values);
    }

    /// Bulk set holdings
    pub fn set_holdings_bulk(&mut self, reg: u16, values: &[u16]) -> Result<(), ErrorKind> {
        set_bulk!(self.holdings, reg, values);
    }

    /// Get a single coil
    pub fn get_coil(&self, reg: u16) -> Result<bool, ErrorKind> {
        get!(self.coils, reg);
    }

    /// Get a single discrete
    pub fn get_discrete(&self, reg: u16) -> Result<bool, ErrorKind> {
        get!(self.discretes, reg);
    }

    /// Get a single input
    pub fn get_input(&self, reg: u16) -> Result<u16, ErrorKind> {
        get!(self.inputs, reg);
    }

    /// Get a single holding
    pub fn get_holding(&self, reg: u16) -> Result<u16, ErrorKind> {
        get!(self.holdings, reg);
    }

    /// Set a single coil
    pub fn set_coil(&mut self, reg: u16, value: bool) -> Result<(), ErrorKind> {
        set!(self.coils, reg, value);
    }

    /// Set a single discrete
    pub fn set_discrete(&mut self, reg: u16, value: bool) -> Result<(), ErrorKind> {
        set!(self.discretes, reg, value);
    }

    /// Set a single input
    pub fn set_input(&mut self, reg: u16, value: u16) -> Result<(), ErrorKind> {
        set!(self.inputs, reg, value);
    }

    /// Set a single holding
    pub fn set_holding(&mut self, reg: u16, value: u16) -> Result<(), ErrorKind> {
        set!(self.holdings, reg, value);
    }

    /// Get two inputs as u32
    ///
    /// Returns 32-bit value (big-endian)
    pub fn get_inputs_as_u32(&self, reg: u16) -> Result<u32, ErrorKind> {
        get_u32!(self.inputs, reg);
    }

    /// Get two holdings as u32
    ///
    /// Returns 32-bit value (big-endian)
    pub fn get_holdings_as_u32(&self, reg: u16) -> Result<u32, ErrorKind> {
        get_u32!(self.holdings, reg);
    }

    /// Set two inputs from u32
    ///
    /// Uses 32-bit value to set two registers (big-endian)
    pub fn set_inputs_from_u32(&mut self, reg: u16, value: u32) -> Result<(), ErrorKind> {
        set_u32!(self.inputs, reg, value);
    }

    /// Set two holdings from u32
    ///
    /// Uses 32-bit value to set two registers (big-endian)
    pub fn set_holdings_from_u32(&mut self, reg: u16, value: u32) -> Result<(), ErrorKind> {
        set_u32!(self.holdings, reg, value);
    }

    /// Get four inputs as u64
    ///
    /// Returns 32-bit value (big-endian)
    pub fn get_inputs_as_u64(&self, reg: u16) -> Result<u64, ErrorKind> {
        get_u64!(self.inputs, reg);
    }

    /// Get four holdings as u64
    ///
    /// Returns 32-bit value (big-endian)
    pub fn get_holdings_as_u64(&self, reg: u16) -> Result<u64, ErrorKind> {
        get_u64!(self.holdings, reg);
    }

    /// Set four inputs from u64
    ///
    /// Uses 32-bit value to set four registers (big-endian)
    pub fn set_inputs_from_u64(&mut self, reg: u16, value: u64) -> Result<(), ErrorKind> {
        set_u64!(self.inputs, reg, value);
    }

    /// Set four holdings from u64
    ///
    /// Uses 32-bit value to set four registers (big-endian)
    pub fn set_holdings_from_u64(&mut self, reg: u16, value: u64) -> Result<(), ErrorKind> {
        set_u64!(self.holdings, reg, value);
    }

    /// Get two input registers as IEEE754 32-bit float
    pub fn get_inputs_as_f32(&self, reg: u16) -> Result<f32, ErrorKind> {
        let i = match self.get_inputs_as_u32(reg) {
            Ok(v) => v,
            Err(v) => return Err(v),
        };
        return Ok(Ieee754::from_bits(i));
    }

    /// Get two holding registers as IEEE754 32-bit float
    pub fn get_holdings_as_f32(&self, reg: u16) -> Result<f32, ErrorKind> {
        let i = match self.get_holdings_as_u32(reg) {
            Ok(v) => v,
            Err(v) => return Err(v),
        };
        return Ok(Ieee754::from_bits(i));
    }

    /// Set IEEE 754 f32 to two input registers
    pub fn set_inputs_from_f32(&mut self, reg: u16, value: f32) -> Result<(), ErrorKind> {
        return self.set_inputs_from_u32(reg, value.bits());
    }

    /// Set IEEE 754 f32 to two holding registers
    pub fn set_holdings_from_f32(&mut self, reg: u16, value: f32) -> Result<(), ErrorKind> {
        return self.set_holdings_from_u32(reg, value.bits());
    }

    /// Get context cell as u8 byte
    ///
    /// 16-bit registers are returned as big-endians
    ///
    /// Offset, for CONTEXT_SIZE = 10000:
    ///
    /// * 0 - 1249: coils as u8
    /// * 1250 - 2499: discretes as u8
    /// * 2500 - 22499: inputs as u8
    /// * 22500 - 42499: holdings as u8
    pub fn get_cell(&self, offset: u16) -> Result<u8, ErrorKind> {
        let bool_ctx_size: usize = CONTEXT_SIZE >> 3;
        let u16_ctx_size: usize = CONTEXT_SIZE << 1;
        if offset < bool_ctx_size as u16 {
            return Ok(get_b_u8(offset * 8, &self.coils));
        }
        if offset < bool_ctx_size as u16 * 2 {
            return Ok(get_b_u8(
                (offset - bool_ctx_size as u16) * 8,
                &self.discretes,
            ));
        }
        if offset < bool_ctx_size as u16 * 2 + u16_ctx_size as u16 {
            return Ok(get_w_u8(
                (offset - bool_ctx_size as u16 * 2) / 2,
                offset % 2 == 0,
                &self.inputs,
            ));
        }
        if offset < bool_ctx_size as u16 * 2 + u16_ctx_size as u16 * 2 {
            return Ok(get_w_u8(
                (offset - (bool_ctx_size as u16 * 2 + u16_ctx_size as u16)) / 2,
                offset % 2 == 0,
                &self.holdings,
            ));
        }
        return Err(ErrorKind::OOBContext);
    }

    /// Set context cell as u8 byte
    ///
    /// 16-bit registers are returned as big-endians
    ///
    /// Offset, for CONTEXT_SIZE = 10000:
    ///
    /// * 0 - 1249: coils as u8
    /// * 1250 - 2499: discretes as u8
    /// * 2500 - 22499: inputs as u8
    /// * 22500- 42499: holdings as u8
    pub fn set_cell(&mut self, offset: u16, value: u8) -> Result<(), ErrorKind> {
        let bool_ctx_size: usize = CONTEXT_SIZE >> 3;
        let u16_ctx_size: usize = CONTEXT_SIZE << 1;
        if offset < bool_ctx_size as u16 {
            return Ok(set_b_u8(offset * 8, value, &mut self.coils));
        }
        if offset < bool_ctx_size as u16 * 2 {
            return Ok(set_b_u8(
                (offset - bool_ctx_size as u16) * 8,
                value,
                &mut self.discretes,
            ));
        }
        if offset < bool_ctx_size as u16 * 2 + u16_ctx_size as u16 {
            return Ok(set_w_u8(
                (offset - bool_ctx_size as u16 * 2) / 2,
                offset % 2 == 0,
                value,
                &mut self.inputs,
            ));
        }
        if offset < bool_ctx_size as u16 * 2 + u16_ctx_size as u16 * 2 {
            return Ok(set_w_u8(
                (offset - (bool_ctx_size as u16 * 2 + u16_ctx_size as u16)) / 2,
                offset % 2 == 0,
                value,
                &mut self.holdings,
            ));
        }
        return Err(ErrorKind::OOBContext);
    }
}

//
// import / export
//
fn get_b_u8(reg_start: u16, reg_context: &[bool]) -> u8 {
    let mut cbyte = 0;
    for i in 0..8 {
        if reg_context[reg_start as usize + i] {
            cbyte = cbyte | 1 << i
        }
    }
    return cbyte;
}

fn set_b_u8(reg_start: u16, value: u8, reg_context: &mut [bool]) {
    let mut b = value;
    for i in 0..8 {
        reg_context[reg_start as usize + i] = b & 1 as u8 == 1;
        b = b >> 1;
    }
}

fn get_w_u8(reg_start: u16, higher: bool, reg_context: &[u16]) -> u8 {
    return match higher {
        true => (reg_context[reg_start as usize] >> 8) as u8,
        false => reg_context[(reg_start as usize)] as u8,
    };
}

fn set_w_u8(reg_start: u16, higher: bool, value: u8, reg_context: &mut [u16]) {
    match higher {
        true => {
            reg_context[reg_start as usize] =
                reg_context[reg_start as usize] & 0x00ff | (value as u16) << 8
        }
        false => {
            reg_context[reg_start as usize] =
                reg_context[reg_start as usize] & 0xff00 | (value as u16)
        }
    };
}

/// A tool to write dumped data back to context
///
/// Can write bytes and chunks (&[u8] slices)
pub struct ModbusContextWriter<'a> {
    curr: u16,
    ctx: &'a mut ModbusContext,
}

impl<'a> ModbusContextWriter<'a> {
    pub fn new(start_offset: u16, ctx: &'a mut ModbusContext) -> Self {
        return Self {
            curr: start_offset,
            ctx: ctx,
        };
    }
    pub fn write(&mut self, value: u8) -> Result<(), ErrorKind> {
        let result = self.ctx.set_cell(self.curr, value);
        if result.is_ok() {
            self.curr = self.curr + 1;
        }
        return result;
    }

    pub fn set_pos(&mut self, offset: u16) {
        self.curr = offset;
    }

    pub fn write_bulk(&mut self, values: &[u8]) -> Result<(), ErrorKind> {
        for v in values {
            let result = self.write(*v);
            if result.is_err() {
                return result;
            }
        }
        return Ok(());
    }
}

pub struct ModbusContextIterator<'a> {
    curr: u16,
    ctx: &'a ModbusContext,
}

impl<'a> Iterator for ModbusContextIterator<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        return match self.ctx.get_cell(self.curr) {
            Ok(v) => {
                self.curr = self.curr + 1;
                Some(v)
            }
            Err(_) => None,
        };
    }
}
