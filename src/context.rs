use ieee754::Ieee754;
use super::super::{ErrorKind, VectorTrait, Mutex, MutexGuard};

pub const CONTEXT_SIZE: usize = 10000;

/// Contains standard Modbus register contexts
pub struct ModbusContext {
    pub coils: [bool; CONTEXT_SIZE],
    pub discretes: [bool; CONTEXT_SIZE],
    pub holdings: [u16; CONTEXT_SIZE],
    pub inputs: [u16; CONTEXT_SIZE],
}

impl ModbusContext {
    fn new() -> Self {
        return ModbusContext {
            coils: [false; CONTEXT_SIZE],
            discretes: [false; CONTEXT_SIZE],
            holdings: [0; CONTEXT_SIZE],
            inputs: [0; CONTEXT_SIZE],
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
/*
/// Dump full Modbus context to Vec<u8>
pub fn dump() -> Option<Vec<u8>> {
    let ctx = lock_mutex!(CONTEXT);
    return dump_locked(&ctx);
}

/// Dump full Modbus context when it's locked
pub fn dump_locked(context: &MutexGuard<ModbusContext>) -> Option<Vec<u8>> {
    let mut data: Vec<u8> = Vec::new(&mut alloc_stack!([u8; CONTEXT_SIZE * 9 / 2]));
    data.push_all(&mut get_bools_as_u8(0, CONTEXT_SIZE as u16, &context.coils).unwrap());
    data.push_all(&mut get_bools_as_u8(0, CONTEXT_SIZE as u16, &context.discretes).unwrap());
    data.push_all(&mut get_regs_as_u8(0, CONTEXT_SIZE as u16, &context.holdings).unwrap());
    data.push_all(&mut get_regs_as_u8(0, CONTEXT_SIZE as u16, &context.inputs).unwrap());
    return data;
}

/// Restore full Modbus context from Vec<u8>
pub fn restore(data: &Vec<u8>) -> Result<(), ErrorKind> {
    let mut ctx = lock_mutex!(CONTEXT);
    return restore_locked(data, &mut ctx);
}

/// Restore full Modbus context when it's locked
pub fn restore_locked(
    data: &Vec<u8>,
    context: &mut MutexGuard<ModbusContext>,
) -> Result<(), ErrorKind> {
    let bool_size = CONTEXT_SIZE / 8;
    let reg_size = CONTEXT_SIZE * 2;
    if bool_size * 2 + reg_size * 2 != data.len() {
        return Err(ErrorKind::OOBContext);
    }
    let start = 0;
    let end = bool_size;
    let coil_values: Vec<u8> = Vec::from(&data[start..end]);
    match set_bools_from_u8(0, CONTEXT_SIZE as u16, &coil_values, &mut context.coils) {
        Ok(_) => {}
        Err(e) => return Err(e),
    };
    let start = start + bool_size;
    let end = end + bool_size;
    let discrete_values: Vec<u8> = Vec::from(&data[start..end]);
    match set_bools_from_u8(
        0,
        CONTEXT_SIZE as u16,
        &discrete_values,
        &mut context.discretes,
    ) {
        Ok(_) => {}
        Err(e) => return Err(e),
    };
    let start = start + bool_size;
    let end = end + reg_size;
    let holding_values: Vec<u8> = Vec::from(&data[start..end]);
    match set_regs_from_u8(0, &holding_values, &mut context.holdings) {
        Ok(_) => {}
        Err(e) => return Err(e),
    };
    let start = start + reg_size;
    let end = end + reg_size;
    let holding_values: Vec<u8> = Vec::from(&data[start..end]);
    match set_regs_from_u8(0, &holding_values, &mut context.inputs) {
        Ok(_) => {}
        Err(e) => return Err(e),
    };
    return Ok(());
}*/
//
// clear
//

/// Clear all coils
pub fn coil_clear_all() {
    with_mut_context(&|context| {
        for i in 0..CONTEXT_SIZE {
            context.coils[i] = false;
        }
    });
}

/// Clear all discrete inputs
pub fn discrete_clear_all() {
    with_mut_context(&|context| {
        for i in 0..CONTEXT_SIZE {
            context.discretes[i] = false;
        }
    });
}

/// Clear all holding registers
pub fn holding_clear_all() {
    with_mut_context(&|context| {
        for i in 0..CONTEXT_SIZE {
            context.holdings[i] = 0;
        }
    });
}

/// Clear all input registers
pub fn input_clear_all() {
    with_mut_context(&|context| {
        for i in 0..CONTEXT_SIZE {
            context.inputs[i] = 0;
        }
    });
}

/// Clear the whole Modbus context
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
pub fn get_bools_as_u8<V: VectorTrait<u8>>(
    reg: u16,
    count: u16,
    reg_context: &[bool; CONTEXT_SIZE],
    result: &mut V,
) -> Result<(), ErrorKind> {
    if count > 250 {
        return Err(ErrorKind::OOBContext);
    }
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
