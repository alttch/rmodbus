use super::super::{ErrorKind, Mutex, MutexGuard, VectorTrait};
use ieee754::Ieee754;

pub const CONTEXT_SIZE: usize = 10000;

/// Contains standard Modbus register contexts
pub struct ModbusContext {
    pub coils: [bool; CONTEXT_SIZE],
    pub discretes: [bool; CONTEXT_SIZE],
    pub inputs: [u16; CONTEXT_SIZE],
    pub holdings: [u16; CONTEXT_SIZE],
}

impl ModbusContext {
    fn new() -> Self {
        return ModbusContext {
            coils: [false; CONTEXT_SIZE],
            discretes: [false; CONTEXT_SIZE],
            inputs: [0; CONTEXT_SIZE],
            holdings: [0; CONTEXT_SIZE],
        };
    }
}

lazy_static! {
    /// Static modbus context storage
    ///
    /// To make everything fast and simple, context is fixed size static array.
    ///
    /// The array contents are protected by mutex to avoid situations where partially-written data
    /// (e.g. during bulk writes or setting 32-bit variables, where more than 1 register is
    /// affected).
    ///
    /// High-level API functions deal with context automatically. To call low-level function,
    /// context must be unlocked manually.
    pub static ref CONTEXT: Mutex<ModbusContext> = Mutex::new(ModbusContext::new());
}

//
// helpers
//

/// Lock context read-only and call a sub-function
///
/// Much faster then high-level API functions, but usually a bit slower than locking context
/// manually.
///
/// Example:
///
///```
///use rmodbus::server::context::with_context;
///
///fn print_coils() {
///    with_context(&|context| {
///        for i in 0..20 {
///            println!("{}", context.coils[i]);
///        }
///    });
///  }
///```
pub fn with_context(f: &dyn Fn(&MutexGuard<ModbusContext>)) {
    let ctx = lock_mutex!(CONTEXT);
    f(&ctx);
}

/// Lock context read-write and call a sub-function
///
/// Much faster then high-level API functions, but usually a bit slower than locking context
/// manually.
///
/// Example:
///
///```
///use rmodbus::server::context::with_mut_context;
///
///
///fn erase_coils(from: usize, to: usize) {
///    with_mut_context(&|context| {
///        for i in from..to {
///            context.coils[i] = false;
///        }
///    });
///}
///```
pub fn with_mut_context(f: &dyn Fn(&mut MutexGuard<ModbusContext>)) {
    let mut ctx = lock_mutex!(CONTEXT);
    f(&mut ctx);
}

//
// import / export
//
fn get_b_u8(reg_start: u16, reg_context: &[bool; CONTEXT_SIZE]) -> u8 {
    let mut cbyte = 0;
    for i in 0..8 {
        if reg_context[reg_start as usize + i] {
            cbyte = cbyte | 1 << i
        }
    }
    return cbyte;
}

fn set_b_u8(reg_start: u16, value: u8, reg_context: &mut [bool; CONTEXT_SIZE]) {
    let mut b = value;
    for i in 0..8 {
        reg_context[reg_start as usize + i] = b & 1 as u8 == 1;
        b = b >> 1;
    }
}

fn get_w_u8(reg_start: u16, higher: bool, reg_context: &[u16; CONTEXT_SIZE]) -> u8 {
    return match higher {
        true => (reg_context[reg_start as usize] >> 8) as u8,
        false => reg_context[(reg_start as usize)] as u8,
    };
}

fn set_w_u8(reg_start: u16, higher: bool, value: u8, reg_context: &mut [u16; CONTEXT_SIZE]) {
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
pub fn get_context_cell(offset: u16, ctx: &MutexGuard<ModbusContext>) -> Result<u8, ErrorKind> {
    let bool_ctx_size: usize = CONTEXT_SIZE >> 3;
    let u16_ctx_size: usize = CONTEXT_SIZE << 1;
    if offset < bool_ctx_size as u16 {
        return Ok(get_b_u8(offset * 8, &ctx.coils));
    }
    if offset < (bool_ctx_size as u16) << 1 {
        return Ok(get_b_u8((offset - 1250) * 8, &ctx.discretes));
    }
    if offset < bool_ctx_size as u16 * 2 + u16_ctx_size as u16 {
        return Ok(get_w_u8((offset - 2500) / 2, offset % 2 == 0, &ctx.inputs));
    }
    if offset < bool_ctx_size as u16 * 2 + u16_ctx_size as u16 * 2 {
        return Ok(get_w_u8(
            (offset - 22500) / 2,
            offset % 2 == 0,
            &ctx.holdings,
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
/// * 22500 - 42499: holdings as u8
pub fn set_context_cell(
    offset: u16,
    value: u8,
    ctx: &mut MutexGuard<ModbusContext>,
) -> Result<(), ErrorKind> {
    if offset < 1250 {
        return Ok(set_b_u8(offset * 8, value, &mut ctx.coils));
    }
    if offset < 2500 {
        return Ok(set_b_u8((offset - 1250) * 8, value, &mut ctx.discretes));
    }
    if offset < 22500 {
        return Ok(set_w_u8(
            (offset - 2500) / 2,
            offset % 2 == 0,
            value,
            &mut ctx.inputs,
        ));
    }
    if offset < 42500 {
        return Ok(set_w_u8(
            (offset - 22500) / 2,
            offset % 2 == 0,
            value,
            &mut ctx.holdings,
        ));
    }
    return Err(ErrorKind::OOBContext);
}

/// A tool to write dumped data back to context
///
/// Can write bytes and chunks (&[u8] slices)
pub struct ModbusContextWriter {
    curr: u16,
}

impl ModbusContextWriter {
    pub fn new(start_offset: u16) -> Self {
        return Self { curr: start_offset };
    }
    pub fn write(
        &mut self,
        value: u8,
        ctx: &mut MutexGuard<ModbusContext>,
    ) -> Result<(), ErrorKind> {
        let result = set_context_cell(self.curr, value, ctx);
        if result.is_ok() {
            self.curr = self.curr + 1;
        }
        return result;
    }

    pub fn write_bulk(
        &mut self,
        values: &[u8],
        ctx: &mut MutexGuard<ModbusContext>,
    ) -> Result<(), ErrorKind> {
        for v in values {
            let result = self.write(*v, ctx);
            if result.is_err() {
                return result;
            }
        }
        return Ok(());
    }
}

pub struct ModbusContextIterator<'a> {
    curr: u16,
    ctx: &'a MutexGuard<'a, ModbusContext>,
}

impl<'a> Iterator for ModbusContextIterator<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        return match get_context_cell(self.curr, self.ctx) {
            Ok(v) => {
                self.curr = self.curr + 1;
                Some(v)
            }
            Err(_) => None,
        };
    }
}

/// Iterate Modbus context as u8
///
/// Useful for dump creation. To restore dump back, use "set_context_cell"
/// or ModbusContextWriter::new()
///
///```ignore
///let ctx = CONTEXT.lock().unwrap();
///for value in context_iter(&ctx) {
///    // store value somewhere
///}
pub fn context_iter<'a>(ctx: &'a MutexGuard<ModbusContext>) -> ModbusContextIterator<'a> {
    return ModbusContextIterator { curr: 0, ctx: ctx };
}

//
// clear, used for tests only, probably never required on production
//

pub fn coil_clear_all() {
    with_mut_context(&|context| {
        for i in 0..CONTEXT_SIZE {
            context.coils[i] = false;
        }
    });
}

pub fn discrete_clear_all() {
    with_mut_context(&|context| {
        for i in 0..CONTEXT_SIZE {
            context.discretes[i] = false;
        }
    });
}

pub fn holding_clear_all() {
    with_mut_context(&|context| {
        for i in 0..CONTEXT_SIZE {
            context.holdings[i] = 0;
        }
    });
}

pub fn input_clear_all() {
    with_mut_context(&|context| {
        for i in 0..CONTEXT_SIZE {
            context.inputs[i] = 0;
        }
    });
}

pub fn clear_all() {
    with_mut_context(&|context| {
        for i in 0..CONTEXT_SIZE {
            context.coils[i] = false;
            context.discretes[i] = false;
            context.holdings[i] = 0;
            context.inputs[i] = 0;
        }
    });
}

//
// get / set with context
//

/// Get 16-bit registers as Vec<u8>
///
/// Useful for import / export and external API calls
/// Note: Vec is always appended
pub fn get_regs_as_u8<V: VectorTrait<u8>>(
    reg: u16,
    count: u16,
    reg_context: &[u16; CONTEXT_SIZE],
    result: &mut V,
) -> Result<(), ErrorKind> {
    let reg_to = reg as usize + count as usize;
    if reg_to > CONTEXT_SIZE {
        return Err(ErrorKind::OOBContext);
    }
    for c in reg as usize..reg_to {
        if result.add((reg_context[c] >> 8) as u8).is_err() {
            return Err(ErrorKind::OOB);
        }
        if result.add(reg_context[c] as u8).is_err() {
            return Err(ErrorKind::OOB);
        }
    }
    return Ok(());
}

/// Set 16-bit registers from &Vec<u8>
///
/// Useful for import / export and external API calls
pub fn set_regs_from_u8(
    reg: u16,
    values: &[u8],
    reg_context: &mut [u16; CONTEXT_SIZE],
) -> Result<(), ErrorKind> {
    let len = values.len();
    if reg as usize + len / 2 > CONTEXT_SIZE {
        return Err(ErrorKind::OOBContext);
    }
    let mut i = 0;
    let mut creg = reg as usize;
    while i < len {
        reg_context[creg] = u16::from_be_bytes([
            values[i],
            match i + 1 < len {
                true => values[i + 1],
                false => return Err(ErrorKind::OOB),
            },
        ]);
        i += 2;
        creg += 1;
    }
    return Ok(());
}

/// Get coils as Vec<u8>
///
/// Useful for import / export and external API calls
/// Note: Vec is always appended
pub fn get_bools_as_u8<V: VectorTrait<u8>>(
    reg: u16,
    count: u16,
    reg_context: &[bool; CONTEXT_SIZE],
    result: &mut V,
) -> Result<(), ErrorKind> {
    let reg_to = reg as usize + count as usize;
    if reg_to > CONTEXT_SIZE {
        return Err(ErrorKind::OOBContext);
    }
    let mut creg = reg as usize;
    while creg < reg_to {
        let mut cbyte = 0;
        for i in 0..8 {
            if reg_context[creg] {
                cbyte = cbyte | 1 << i
            }
            creg += 1;
            if creg >= reg_to {
                break;
            }
        }
        if result.add(cbyte).is_err() {
            return Err(ErrorKind::OOBContext);
        };
    }
    return Ok(());
}

/// Set coils from &Vec<u8>
///
/// Useful for import / export and external API calls
///
/// As coils are packed in u8, parameter *count* specifies how many coils are actually needed to
/// set, extra bits are ignored
pub fn set_bools_from_u8(
    reg: u16,
    count: u16,
    values: &[u8],
    reg_context: &mut [bool; CONTEXT_SIZE],
) -> Result<(), ErrorKind> {
    let reg_to = reg as usize + count as usize;
    if reg_to > CONTEXT_SIZE {
        return Err(ErrorKind::OOBContext);
    }
    let mut creg = reg as usize;
    let mut cbyte = 0;
    let mut cnt = 0;
    let len = values.len();
    while creg < reg_to && cnt < count {
        if cbyte >= len {
            return Err(ErrorKind::OOB);
        }
        let mut b: u8 = values[cbyte];
        for _ in 0..8 {
            reg_context[creg] = b & 1 == 1;
            b = b >> 1;
            creg = creg + 1;
            cnt = cnt + 1;
            if cnt == count || creg == reg_to {
                break;
            }
        }
        cbyte = cbyte + 1;
    }
    return Ok(());
}

/// Bulk get
///
/// Can get the coils (Vec<bool>) or 16-bit registers (Vec<u16>)
/// Note: Vec is always appended
pub fn get_bulk<T: Copy, V: VectorTrait<T>>(
    reg: u16,
    count: u16,
    reg_context: &[T; CONTEXT_SIZE],
    result: &mut V,
) -> Result<(), ErrorKind> {
    let reg_to = reg as usize + count as usize;
    if reg_to > CONTEXT_SIZE {
        return Err(ErrorKind::OOBContext);
    }
    if result.add_bulk(&reg_context[reg as usize..reg_to]).is_err() {
        return Err(ErrorKind::OOBContext);
    }
    return Ok(());
}

/// Bulk set
///
/// Can set the coils from Vec<bool> or 16-bit registers from Vec<u16>
pub fn set_bulk<T: Copy>(
    reg: u16,
    values: &[T],
    reg_context: &mut [T; CONTEXT_SIZE],
) -> Result<(), ErrorKind> {
    if reg as usize + values.len() > CONTEXT_SIZE {
        return Err(ErrorKind::OOBContext);
    }
    for (i, value) in values.iter().enumerate() {
        reg_context[reg as usize + i] = *value;
    }
    return Ok(());
}

/// Get a single register
///
/// Get coil as bool or 16-bit reg as u16
pub fn get<T: Copy>(reg: u16, reg_context: &[T; CONTEXT_SIZE]) -> Result<T, ErrorKind> {
    if reg as usize >= CONTEXT_SIZE {
        return Err(ErrorKind::OOBContext);
    }
    return Ok(reg_context[reg as usize]);
}

/// Set a single register
///
/// Set coil from bool or 16-bit reg from u16
pub fn set<T>(reg: u16, value: T, reg_context: &mut [T; CONTEXT_SIZE]) -> Result<(), ErrorKind> {
    if reg as usize >= CONTEXT_SIZE {
        return Err(ErrorKind::OOBContext);
    }
    reg_context[reg as usize] = value;
    return Ok(());
}

/// Get two 16-bit registers as u32
///
/// Returns 32-bit value (big-endian)
pub fn get_u32(reg: u16, reg_context: &[u16; CONTEXT_SIZE]) -> Result<u32, ErrorKind> {
    let w1 = match get(reg, reg_context) {
        Ok(v) => v,
        Err(v) => return Err(v),
    };
    let w2 = match get(reg + 1, reg_context) {
        Ok(v) => v,
        Err(v) => return Err(v),
    };
    return Ok(((w1 as u32) << 16) + w2 as u32);
}

/// Set two 16-bit registers from u32
///
/// Uses 32-bit value to set two registers (big-endian)
pub fn set_u32(
    reg: u16,
    value: u32,
    reg_context: &mut [u16; CONTEXT_SIZE],
) -> Result<(), ErrorKind> {
    if reg as usize + 2 > CONTEXT_SIZE {
        return Err(ErrorKind::OOBContext);
    }
    reg_context[reg as usize] = (value >> 16) as u16;
    reg_context[reg as usize + 1] = value as u16;
    return Ok(());
}

/// Set multiple 16-bit registers from Vec<u32>
///
/// Uses Vec of 32-bit values to set the registers (big-endian)
pub fn set_u32_bulk(
    reg: u16,
    values: &[u32],
    reg_context: &mut [u16; CONTEXT_SIZE],
) -> Result<(), ErrorKind> {
    if reg as usize + values.len() * 2 >= CONTEXT_SIZE {
        return Err(ErrorKind::OOBContext);
    }
    let mut reg_c = reg;
    for value in values {
        reg_context[reg_c as usize] = (value >> 16) as u16;
        reg_context[reg_c as usize + 1] = *value as u16;
        reg_c = reg_c + 2;
    }
    return Ok(());
}

/// Get two 16-bit registers as IEEE754 32-bit float
pub fn get_f32(reg: u16, reg_context: &[u16; CONTEXT_SIZE]) -> Result<f32, ErrorKind> {
    let i = match get_u32(reg, reg_context) {
        Ok(v) => v,
        Err(v) => return Err(v),
    };
    return Ok(Ieee754::from_bits(i));
}

/// Set IEEE 754 f32 to two 16-bit registers
pub fn set_f32(
    reg: u16,
    value: f32,
    reg_context: &mut [u16; CONTEXT_SIZE],
) -> Result<(), ErrorKind> {
    return set_u32(reg, value.bits(), reg_context);
}

/// Set multiple 16-bit registers from Vec<f32> as IEEE754 32-bit floats
pub fn set_f32_bulk(
    reg: u16,
    values: &[f32],
    reg_context: &mut [u16; CONTEXT_SIZE],
) -> Result<(), ErrorKind> {
    if reg as usize + values.len() * 2 >= CONTEXT_SIZE {
        return Err(ErrorKind::OOBContext);
    }
    let mut reg_c = reg;
    for value in values {
        let b = value.bits();
        reg_context[reg_c as usize] = (b >> 16) as u16;
        reg_context[reg_c as usize + 1] = b as u16;
        reg_c = reg_c + 2;
    }
    return Ok(());
}

//
// coils functions
//

pub fn coil_get_bulk<V: VectorTrait<bool>>(
    reg: u16,
    count: u16,
    result: &mut V,
) -> Result<(), ErrorKind> {
    let context = lock_mutex!(CONTEXT);
    return get_bulk(reg, count, &context.coils, result);
}

pub fn coil_set_bulk(reg: u16, coils: &[bool]) -> Result<(), ErrorKind> {
    let mut context = lock_mutex!(CONTEXT);
    return set_bulk(reg, coils, &mut context.coils);
}

pub fn coil_get(reg: u16) -> Result<bool, ErrorKind> {
    let context = lock_mutex!(CONTEXT);
    return get(reg, &context.coils);
}

pub fn coil_set(reg: u16, value: bool) -> Result<(), ErrorKind> {
    let mut context = lock_mutex!(CONTEXT);
    return set(reg, value, &mut context.coils);
}
//
// discretes functions
//

pub fn discrete_get_bulk<V: VectorTrait<bool>>(
    reg: u16,
    count: u16,
    result: &mut V,
) -> Result<(), ErrorKind> {
    let context = lock_mutex!(CONTEXT);
    return get_bulk(reg, count, &context.discretes, result);
}

pub fn discrete_set_bulk(reg: u16, discretes: &[bool]) -> Result<(), ErrorKind> {
    let mut context = lock_mutex!(CONTEXT);
    return set_bulk(reg, discretes, &mut context.discretes);
}

pub fn discrete_get(reg: u16) -> Result<bool, ErrorKind> {
    let context = lock_mutex!(CONTEXT);
    return get(reg, &context.discretes);
}

pub fn discrete_set(reg: u16, value: bool) -> Result<(), ErrorKind> {
    let mut context = lock_mutex!(CONTEXT);
    return set(reg, value, &mut context.discretes);
}

//
// holdings functions
//
pub fn holding_get(reg: u16) -> Result<u16, ErrorKind> {
    let context = lock_mutex!(CONTEXT);
    return get(reg, &context.holdings);
}

pub fn holding_get_bulk<V: VectorTrait<u16>>(
    reg: u16,
    count: u16,
    result: &mut V,
) -> Result<(), ErrorKind> {
    let context = lock_mutex!(CONTEXT);
    return get_bulk(reg, count, &context.holdings, result);
}

pub fn holding_get_u32(reg: u16) -> Result<u32, ErrorKind> {
    let context = lock_mutex!(CONTEXT);
    return get_u32(reg, &context.holdings);
}

pub fn holding_get_f32(reg: u16) -> Result<f32, ErrorKind> {
    let context = lock_mutex!(CONTEXT);
    return get_f32(reg, &context.holdings);
}

pub fn holding_set(reg: u16, value: u16) -> Result<(), ErrorKind> {
    let mut context = lock_mutex!(CONTEXT);
    return set(reg, value, &mut context.holdings);
}

pub fn holding_set_bulk(reg: u16, holdings: &[u16]) -> Result<(), ErrorKind> {
    let mut context = lock_mutex!(CONTEXT);
    return set_bulk(reg, &holdings, &mut context.holdings);
}

pub fn holding_set_u32_bulk(reg: u16, values: &[u32]) -> Result<(), ErrorKind> {
    let mut context = lock_mutex!(CONTEXT);
    return set_u32_bulk(reg, values, &mut context.holdings);
}

pub fn holding_set_u32(reg: u16, value: u32) -> Result<(), ErrorKind> {
    let mut context = lock_mutex!(CONTEXT);
    return set_u32(reg, value, &mut context.holdings);
}

pub fn holding_set_f32_bulk(reg: u16, values: &[f32]) -> Result<(), ErrorKind> {
    let mut context = lock_mutex!(CONTEXT);
    return set_f32_bulk(reg, values, &mut context.holdings);
}

pub fn holding_set_f32(reg: u16, value: f32) -> Result<(), ErrorKind> {
    let mut context = lock_mutex!(CONTEXT);
    return set_f32(reg, value, &mut context.holdings);
}

//
// input functions
//
pub fn input_get(reg: u16) -> Result<u16, ErrorKind> {
    let context = lock_mutex!(CONTEXT);
    return get(reg, &context.inputs);
}

pub fn input_get_bulk<V: VectorTrait<u16>>(
    reg: u16,
    count: u16,
    result: &mut V,
) -> Result<(), ErrorKind> {
    let context = lock_mutex!(CONTEXT);
    return get_bulk(reg, count, &context.inputs, result);
}

pub fn input_get_u32(reg: u16) -> Result<u32, ErrorKind> {
    let context = lock_mutex!(CONTEXT);
    return get_u32(reg, &context.inputs);
}

pub fn input_get_f32(reg: u16) -> Result<f32, ErrorKind> {
    let context = lock_mutex!(CONTEXT);
    return get_f32(reg, &context.inputs);
}

pub fn input_set(reg: u16, value: u16) -> Result<(), ErrorKind> {
    let mut context = lock_mutex!(CONTEXT);
    return set(reg, value, &mut context.inputs);
}

pub fn input_set_bulk(reg: u16, inputs: &[u16]) -> Result<(), ErrorKind> {
    let mut context = lock_mutex!(CONTEXT);
    return set_bulk(reg, &inputs, &mut context.inputs);
}

pub fn input_set_u32_bulk(reg: u16, values: &[u32]) -> Result<(), ErrorKind> {
    let mut context = lock_mutex!(CONTEXT);
    return set_u32_bulk(reg, values, &mut context.inputs);
}

pub fn input_set_u32(reg: u16, value: u32) -> Result<(), ErrorKind> {
    let mut context = lock_mutex!(CONTEXT);
    return set_u32(reg, value, &mut context.inputs);
}

pub fn input_set_f32_bulk(reg: u16, values: &[f32]) -> Result<(), ErrorKind> {
    let mut context = lock_mutex!(CONTEXT);
    return set_f32_bulk(reg, values, &mut context.inputs);
}

pub fn input_set_f32(reg: u16, value: f32) -> Result<(), ErrorKind> {
    let mut context = lock_mutex!(CONTEXT);
    return set_f32(reg, value, &mut context.inputs);
}
