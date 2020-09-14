use ieee754::Ieee754;
use spin::{Mutex, MutexGuard};

use fixedvec::FixedVec;

const CONTEXT_SIZE: usize = 10000;

/// Contains standard Modbus register contexts
pub struct ModbusContext {
    pub coils: [bool; CONTEXT_SIZE],
    pub discretes: [bool; CONTEXT_SIZE],
    pub holdings: [u16; CONTEXT_SIZE],
    pub inputs: [u16; CONTEXT_SIZE],
}

/// Default context error
///
/// Returned by all functions. Usually caused when read / write request is out of bounds.
#[derive(Debug, Clone)]
pub struct Error;

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
    let ctx = CONTEXT.lock();
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
    let mut ctx = CONTEXT.lock();
    f(&mut ctx);
}

//
// import / export
//
/*
/// Dump full Modbus context to Vec<u8>
pub fn dump() -> Option<FixedVec<u8>> {
    let ctx = CONTEXT.lock();
    return dump_locked(&ctx);
}

/// Dump full Modbus context when it's locked
pub fn dump_locked(context: &MutexGuard<ModbusContext>) -> Option<FixedVec<u8>> {
    let mut data: FixedVec<u8> = FixedVec::new(&mut alloc_stack!([u8; CONTEXT_SIZE * 9 / 2]));
    data.push_all(&mut get_bools_as_u8(0, CONTEXT_SIZE as u16, &context.coils).unwrap());
    data.push_all(&mut get_bools_as_u8(0, CONTEXT_SIZE as u16, &context.discretes).unwrap());
    data.push_all(&mut get_regs_as_u8(0, CONTEXT_SIZE as u16, &context.holdings).unwrap());
    data.push_all(&mut get_regs_as_u8(0, CONTEXT_SIZE as u16, &context.inputs).unwrap());
    return data;
}

/// Restore full Modbus context from Vec<u8>
pub fn restore(data: &Vec<u8>) -> Result<(), Error> {
    let mut ctx = CONTEXT.lock();
    return restore_locked(data, &mut ctx);
}

/// Restore full Modbus context when it's locked
pub fn restore_locked(
    data: &Vec<u8>,
    context: &mut MutexGuard<ModbusContext>,
) -> Result<(), Error> {
    let bool_size = CONTEXT_SIZE / 8;
    let reg_size = CONTEXT_SIZE * 2;
    if bool_size * 2 + reg_size * 2 != data.len() {
        return Err(Error);
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
pub fn get_regs_as_u8(
    reg: u16,
    count: u16,
    reg_context: &[u16; CONTEXT_SIZE],
    result: &mut FixedVec<u8>,
) -> Result<(), Error> {
    let reg_to = reg as usize + count as usize;
    if reg_to > CONTEXT_SIZE {
        return Err(Error);
    }
    for c in reg as usize..reg_to {
        if result.push((reg_context[c] >> 8) as u8).is_err() {
            return Err(Error);
        }
        if result.push(reg_context[c] as u8).is_err() {
            return Err(Error);
        }
    }
    return Ok(());
}

/// Set 16-bit registers from &Vec<u8>
///
/// Useful for import / export and external API calls
pub fn set_regs_from_u8(
    reg: u16,
    values: &FixedVec<u8>,
    reg_context: &mut [u16; CONTEXT_SIZE],
) -> Result<(), Error> {
    if reg as usize + values.len() / 2 > CONTEXT_SIZE {
        return Err(Error);
    }
    let mut i = 0;
    let mut creg = reg as usize;
    while i < values.len() {
        reg_context[creg] = u16::from_be_bytes([
            *values.get(i).unwrap(),
            match values.get(i + 1) {
                Some(v) => *v,
                None => 0,
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
pub fn get_bools_as_u8(
    reg: u16,
    count: u16,
    reg_context: &[bool; CONTEXT_SIZE],
    result: &mut FixedVec<u8>,
) -> Result<(), Error> {
    if count > 250 {
        return Err(Error);
    }
    let reg_to = reg as usize + count as usize;
    if reg_to > CONTEXT_SIZE {
        return Err(Error);
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
        if result.push(cbyte).is_err() {
            return Err(Error);
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
    values: &FixedVec<u8>,
    reg_context: &mut [bool; CONTEXT_SIZE],
) -> Result<(), Error> {
    let reg_to = reg as usize + count as usize;
    if reg_to > CONTEXT_SIZE {
        return Err(Error);
    }
    let mut creg = reg as usize;
    let mut cbyte = 0;
    let mut cnt = 0;
    while creg < reg_to && cnt < count {
        let mut b: u8 = match values.get(cbyte) {
            Some(v) => *v,
            None => return Err(Error),
        };
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
/// Can get the coils (FixedVec<bool>) or 16-bit registers (FixedVec<u16>)
pub fn get_bulk<T: Copy>(
    reg: u16,
    count: u16,
    reg_context: &[T; CONTEXT_SIZE],
    result: &mut FixedVec<T>,
) -> Result<(), Error> {
    let reg_to = reg as usize + count as usize;
    if reg_to > CONTEXT_SIZE {
        return Err(Error);
    }
    if result.push_all(&reg_context[reg as usize..reg_to]).is_err() {
        return Err(Error);
    }
    return Ok(());
}

/// Bulk set
///
/// Can set the coils from Vec<bool> or 16-bit registers from Vec<u16>
pub fn set_bulk<T: Copy>(
    reg: u16,
    data: &FixedVec<T>,
    reg_context: &mut [T; CONTEXT_SIZE],
) -> Result<(), Error> {
    if reg as usize + data.len() > CONTEXT_SIZE {
        return Err(Error);
    }
    for (i, value) in data.iter().enumerate() {
        reg_context[reg as usize + i] = *value;
    }
    return Ok(());
}

/// Get a single register
///
/// Get coil as bool or 16-bit reg as u16
pub fn get<T: Copy>(reg: u16, reg_context: &[T; CONTEXT_SIZE]) -> Result<T, Error> {
    if reg as usize >= CONTEXT_SIZE {
        return Err(Error);
    }
    return Ok(reg_context[reg as usize]);
}

/// Set a single register
///
/// Set coil from bool or 16-bit reg from u16
pub fn set<T>(reg: u16, value: T, reg_context: &mut [T; CONTEXT_SIZE]) -> Result<(), Error> {
    if reg as usize >= CONTEXT_SIZE {
        return Err(Error);
    }
    reg_context[reg as usize] = value;
    return Ok(());
}

/// Get two 16-bit registers as u32
///
/// Returns 32-bit value (big-endian)
pub fn get_u32(reg: u16, reg_context: &[u16; CONTEXT_SIZE]) -> Result<u32, Error> {
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
pub fn set_u32(reg: u16, value: u32, reg_context: &mut [u16; CONTEXT_SIZE]) -> Result<(), Error> {
    if reg as usize + 2 > CONTEXT_SIZE {
        return Err(Error);
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
    values: &FixedVec<u32>,
    reg_context: &mut [u16; CONTEXT_SIZE],
) -> Result<(), Error> {
    if reg as usize + values.len() * 2 >= CONTEXT_SIZE {
        return Err(Error);
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
pub fn get_f32(reg: u16, reg_context: &[u16; CONTEXT_SIZE]) -> Result<f32, Error> {
    let i = match get_u32(reg, reg_context) {
        Ok(v) => v,
        Err(v) => return Err(v),
    };
    return Ok(Ieee754::from_bits(i));
}

/// Set IEEE 754 f32 to two 16-bit registers
pub fn set_f32(reg: u16, value: f32, reg_context: &mut [u16; CONTEXT_SIZE]) -> Result<(), Error> {
    return set_u32(reg, value.bits(), reg_context);
}

/// Set multiple 16-bit registers from Vec<f32> as IEEE754 32-bit floats
pub fn set_f32_bulk(
    reg: u16,
    values: &FixedVec<f32>,
    reg_context: &mut [u16; CONTEXT_SIZE],
) -> Result<(), Error> {
    if reg as usize + values.len() * 2 >= CONTEXT_SIZE {
        return Err(Error);
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

pub fn coil_get_bulk(reg: u16, count: u16, result: &mut FixedVec<bool>) -> Result<(), Error> {
    let context = CONTEXT.lock();
    return get_bulk(reg, count, &context.coils, result);
}

pub fn coil_set_bulk(reg: u16, coils: &FixedVec<bool>) -> Result<(), Error> {
    let mut context = CONTEXT.lock();
    return set_bulk(reg, coils, &mut context.coils);
}

pub fn coil_get(reg: u16) -> Result<bool, Error> {
    let context = CONTEXT.lock();
    return get(reg, &context.coils);
}

pub fn coil_set(reg: u16, value: bool) -> Result<(), Error> {
    let mut context = CONTEXT.lock();
    return set(reg, value, &mut context.coils);
}
//
// discretes functions
//

pub fn discrete_get_bulk(reg: u16, count: u16, result: &mut FixedVec<bool>) -> Result<(), Error> {
    let context = CONTEXT.lock();
    return get_bulk(reg, count, &context.discretes, result);
}

pub fn discrete_set_bulk(reg: u16, discretes: &FixedVec<bool>) -> Result<(), Error> {
    let mut context = CONTEXT.lock();
    return set_bulk(reg, discretes, &mut context.discretes);
}

pub fn discrete_get(reg: u16) -> Result<bool, Error> {
    let context = CONTEXT.lock();
    return get(reg, &context.discretes);
}

pub fn discrete_set(reg: u16, value: bool) -> Result<(), Error> {
    let mut context = CONTEXT.lock();
    return set(reg, value, &mut context.discretes);
}

//
// holdings functions
//
pub fn holding_get(reg: u16) -> Result<u16, Error> {
    let context = CONTEXT.lock();
    return get(reg, &context.holdings);
}

pub fn holding_get_bulk(reg: u16, count: u16, result: &mut FixedVec<u16>) -> Result<(), Error> {
    let context = CONTEXT.lock();
    return get_bulk(reg, count, &context.holdings, result);
}

pub fn holding_get_u32(reg: u16) -> Result<u32, Error> {
    let context = CONTEXT.lock();
    return get_u32(reg, &context.holdings);
}

pub fn holding_get_f32(reg: u16) -> Result<f32, Error> {
    let context = CONTEXT.lock();
    return get_f32(reg, &context.holdings);
}

pub fn holding_set(reg: u16, value: u16) -> Result<(), Error> {
    let mut context = CONTEXT.lock();
    return set(reg, value, &mut context.holdings);
}

pub fn holding_set_bulk(reg: u16, holdings: &FixedVec<u16>) -> Result<(), Error> {
    let mut context = CONTEXT.lock();
    return set_bulk(reg, &holdings, &mut context.holdings);
}

pub fn holding_set_u32_bulk(reg: u16, values: &FixedVec<u32>) -> Result<(), Error> {
    let mut context = CONTEXT.lock();
    return set_u32_bulk(reg, values, &mut context.holdings);
}

pub fn holding_set_u32(reg: u16, value: u32) -> Result<(), Error> {
    let mut context = CONTEXT.lock();
    return set_u32(reg, value, &mut context.holdings);
}

pub fn holding_set_f32_bulk(reg: u16, values: &FixedVec<f32>) -> Result<(), Error> {
    let mut context = CONTEXT.lock();
    return set_f32_bulk(reg, values, &mut context.holdings);
}

pub fn holding_set_f32(reg: u16, value: f32) -> Result<(), Error> {
    let mut context = CONTEXT.lock();
    return set_f32(reg, value, &mut context.holdings);
}

//
// input functions
//
pub fn input_get(reg: u16) -> Result<u16, Error> {
    let context = CONTEXT.lock();
    return get(reg, &context.inputs);
}

pub fn input_get_bulk(reg: u16, count: u16, result: &mut FixedVec<u16>) -> Result<(), Error> {
    let context = CONTEXT.lock();
    return get_bulk(reg, count, &context.inputs, result);
}

pub fn input_get_u32(reg: u16) -> Result<u32, Error> {
    let context = CONTEXT.lock();
    return get_u32(reg, &context.inputs);
}

pub fn input_get_f32(reg: u16) -> Result<f32, Error> {
    let context = CONTEXT.lock();
    return get_f32(reg, &context.inputs);
}

pub fn input_set(reg: u16, value: u16) -> Result<(), Error> {
    let mut context = CONTEXT.lock();
    return set(reg, value, &mut context.inputs);
}

pub fn input_set_bulk(reg: u16, inputs: &FixedVec<u16>) -> Result<(), Error> {
    let mut context = CONTEXT.lock();
    return set_bulk(reg, &inputs, &mut context.inputs);
}

pub fn input_set_u32_bulk(reg: u16, values: &FixedVec<u32>) -> Result<(), Error> {
    let mut context = CONTEXT.lock();
    return set_u32_bulk(reg, values, &mut context.inputs);
}

pub fn input_set_u32(reg: u16, value: u32) -> Result<(), Error> {
    let mut context = CONTEXT.lock();
    return set_u32(reg, value, &mut context.inputs);
}

pub fn input_set_f32_bulk(reg: u16, values: &FixedVec<f32>) -> Result<(), Error> {
    let mut context = CONTEXT.lock();
    return set_f32_bulk(reg, values, &mut context.inputs);
}

pub fn input_set_f32(reg: u16, value: f32) -> Result<(), Error> {
    let mut context = CONTEXT.lock();
    return set_f32(reg, value, &mut context.inputs);
}
#[cfg(test)]
mod tests {
    use super::*;
    //use rand::Rng;

    #[test]
    fn test_read_coils_as_bytes_oob() {
        let mut preallocated = alloc_stack!([bool; CONTEXT_SIZE + 1]);
        let mut result = FixedVec::new(&mut preallocated);
        match coil_get_bulk(0, CONTEXT_SIZE as u16 + 1, &mut result) {
            Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
            Err(_) => assert!(true),
        }
        match coil_get_bulk(CONTEXT_SIZE as u16, 1, &mut result) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        coil_get_bulk((CONTEXT_SIZE - 1) as u16, 1, &mut result).unwrap();
        match coil_get_bulk(CONTEXT_SIZE as u16 - 1, 2, &mut result) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        coil_get((CONTEXT_SIZE - 1) as u16).unwrap();
        match coil_get(CONTEXT_SIZE as u16) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        coil_set((CONTEXT_SIZE - 1) as u16, true).unwrap();
        match coil_set(CONTEXT_SIZE as u16, true) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_coil_get_set_bulk() {
        let mut data_mem = alloc_stack!([bool; CONTEXT_SIZE]);
        let mut data = FixedVec::new(&mut data_mem);
        let mut result_mem = alloc_stack!([bool; CONTEXT_SIZE]);
        let mut result = FixedVec::new(&mut result_mem);
        data.push_all(&[true; 2]).unwrap();
        coil_set_bulk(5, &data).unwrap();
        coil_get_bulk(5, 2, &mut result).unwrap();
        assert_eq!(result, data);

        data.clear();
        result.clear();

        data.push_all(&[true; 18]).unwrap();
        coil_set_bulk(25, &data).unwrap();
        coil_get_bulk(25, 18, &mut result).unwrap();
        assert_eq!(result, data);

        coil_set(28, true).unwrap();
        assert_eq!(coil_get(28).unwrap(), true);
        coil_set(28, false).unwrap();
        assert_eq!(coil_get(28).unwrap(), false);
    }

    #[test]
    fn test_read_discretes_as_bytes_oob() {
        let mut preallocated = alloc_stack!([bool; CONTEXT_SIZE + 1]);
        let mut result = FixedVec::new(&mut preallocated);
        match discrete_get_bulk(0, CONTEXT_SIZE as u16 + 1, &mut result) {
            Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
            Err(_) => assert!(true),
        }
        match discrete_get_bulk(CONTEXT_SIZE as u16, 1, &mut result) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        discrete_get_bulk((CONTEXT_SIZE - 1) as u16, 1, &mut result).unwrap();
        match discrete_get_bulk(CONTEXT_SIZE as u16 - 1, 2, &mut result) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        discrete_get((CONTEXT_SIZE - 1) as u16).unwrap();
        match discrete_get(CONTEXT_SIZE as u16) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        discrete_set((CONTEXT_SIZE - 1) as u16, true).unwrap();
        match discrete_set(CONTEXT_SIZE as u16, true) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_discrete_get_set_bulk() {
        let mut data_mem = alloc_stack!([bool; CONTEXT_SIZE]);
        let mut data = FixedVec::new(&mut data_mem);
        let mut result_mem = alloc_stack!([bool; CONTEXT_SIZE]);
        let mut result = FixedVec::new(&mut result_mem);
        data.push_all(&[true; 2]).unwrap();
        discrete_set_bulk(5, &data).unwrap();
        discrete_get_bulk(5, 2, &mut result).unwrap();
        assert_eq!(result, data);

        data.clear();
        result.clear();

        data.push_all(&[true; 18]).unwrap();
        discrete_set_bulk(25, &data).unwrap();
        discrete_get_bulk(25, 18, &mut result).unwrap();
        assert_eq!(result, data);

        discrete_set(28, true).unwrap();
        assert_eq!(discrete_get(28).unwrap(), true);
        discrete_set(28, false).unwrap();
        assert_eq!(discrete_get(28).unwrap(), false);
    }

    #[test]
    fn test_read_holdings_as_bytes_oob() {
        let mut preallocated = alloc_stack!([u16; CONTEXT_SIZE + 1]);
        let mut result = FixedVec::new(&mut preallocated);
        match holding_get_bulk(0, CONTEXT_SIZE as u16 + 1, &mut result) {
            Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
            Err(_) => assert!(true),
        }
        match holding_get_bulk(CONTEXT_SIZE as u16, 1, &mut result) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        holding_get_bulk((CONTEXT_SIZE - 1) as u16, 1, &mut result).unwrap();
        match holding_get_bulk(CONTEXT_SIZE as u16 - 1, 2, &mut result) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        holding_get((CONTEXT_SIZE - 1) as u16).unwrap();
        match holding_get(CONTEXT_SIZE as u16) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        holding_set((CONTEXT_SIZE - 1) as u16, 0x55).unwrap();
        match holding_set(CONTEXT_SIZE as u16, 0x55) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        match holding_set_u32((CONTEXT_SIZE - 1) as u16, 0x55) {
            Ok(_) => assert!(false, "oob failed MAX u32"),
            Err(_) => assert!(true),
        }
        holding_set_u32((CONTEXT_SIZE - 2) as u16, 0x9999).unwrap();
    }

    #[test]
    fn test_get_holding_set_bulk() {
        let mut data_mem = alloc_stack!([u16; CONTEXT_SIZE]);
        let mut data = FixedVec::new(&mut data_mem);
        let mut result_mem = alloc_stack!([u16; CONTEXT_SIZE]);
        let mut result = FixedVec::new(&mut result_mem);

        holding_clear_all();

        data.push_all(&[0x77; 2]).unwrap();
        holding_set_bulk(5, &data).unwrap();
        holding_get_bulk(5, 2, &mut result).unwrap();
        assert_eq!(result, data);

        data.clear();
        result.clear();

        data.push_all(&[0x33; 18]).unwrap();
        holding_set_bulk(25, &data).unwrap();
        holding_get_bulk(25, 18, &mut result).unwrap();
        assert_eq!(result, data);

        holding_set(28, 99).unwrap();
        assert_eq!(holding_get(28).unwrap(), 99);
        holding_set(28, 95).unwrap();
        assert_eq!(holding_get(28).unwrap(), 95);
        holding_set_u32(1000, 1234567).unwrap();
        assert_eq!(holding_get_u32(1000).unwrap(), 1234567);
    }

    #[test]
    fn test_get_holding_set_u32() {
        let mut data_mem = alloc_stack!([u32; 2]);
        let mut data = FixedVec::new(&mut data_mem);

        data.push_all(&[1234567, 8901234]).unwrap();
        holding_set_u32_bulk(1002, &data).unwrap();

        assert_eq!(holding_get_u32(1002).unwrap(), 1234567);
        assert_eq!(holding_get_u32(1004).unwrap(), 8901234);

        holding_set_u32(900, 3412345).unwrap();
        assert_eq!(holding_get_u32(900).unwrap(), 3412345);
    }

    #[test]
    fn test_get_holding_set_f32() {
        let mut data_mem = alloc_stack!([f32; 2]);
        let mut data = FixedVec::new(&mut data_mem);

        data.push_all(&[1234.567, 890.1234]).unwrap();

        holding_set_f32_bulk(2002, &data).unwrap();
        assert_eq!(holding_get_f32(2002).unwrap(), 1234.567);
        assert_eq!(holding_get_f32(2004).unwrap(), 890.1234);

        holding_set_f32(2000, 1234.567).unwrap();
        assert_eq!(holding_get_f32(2000).unwrap(), 1234.567);
    }

    #[test]
    fn test_read_inputs_as_bytes_oob() {
        let mut preallocated = alloc_stack!([u16; CONTEXT_SIZE + 1]);
        let mut result = FixedVec::new(&mut preallocated);
        match input_get_bulk(0, CONTEXT_SIZE as u16 + 1, &mut result) {
            Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
            Err(_) => assert!(true),
        }
        match input_get_bulk(CONTEXT_SIZE as u16, 1, &mut result) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        input_get_bulk((CONTEXT_SIZE - 1) as u16, 1, &mut result).unwrap();
        match input_get_bulk(CONTEXT_SIZE as u16 - 1, 2, &mut result) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        input_get((CONTEXT_SIZE - 1) as u16).unwrap();
        match input_get(CONTEXT_SIZE as u16) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        input_set((CONTEXT_SIZE - 1) as u16, 0x55).unwrap();
        match input_set(CONTEXT_SIZE as u16, 0x55) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        match input_set_u32((CONTEXT_SIZE - 1) as u16, 0x55) {
            Ok(_) => assert!(false, "oob failed MAX u32"),
            Err(_) => assert!(true),
        }
        input_set_u32((CONTEXT_SIZE - 2) as u16, 0x9999).unwrap();
    }

    #[test]
    fn test_get_input_set_bulk() {
        let mut data_mem = alloc_stack!([u16; CONTEXT_SIZE]);
        let mut data = FixedVec::new(&mut data_mem);
        let mut result_mem = alloc_stack!([u16; CONTEXT_SIZE]);
        let mut result = FixedVec::new(&mut result_mem);

        input_clear_all();

        data.push_all(&[0x77; 2]).unwrap();
        input_set_bulk(5, &data).unwrap();
        input_get_bulk(5, 2, &mut result).unwrap();
        assert_eq!(result, data);

        data.clear();
        result.clear();

        data.push_all(&[0x33; 18]).unwrap();
        input_set_bulk(25, &data).unwrap();
        input_get_bulk(25, 18, &mut result).unwrap();
        assert_eq!(result, data);

        input_set(28, 99).unwrap();
        assert_eq!(input_get(28).unwrap(), 99);
        input_set(28, 95).unwrap();
        assert_eq!(input_get(28).unwrap(), 95);
        input_set_u32(1000, 1234567).unwrap();
        assert_eq!(input_get_u32(1000).unwrap(), 1234567);
    }

    #[test]
    fn test_get_input_set_u32() {
        let mut data_mem = alloc_stack!([u32; 2]);
        let mut data = FixedVec::new(&mut data_mem);

        data.push_all(&[1234567, 8901234]).unwrap();
        input_set_u32_bulk(1002, &data).unwrap();

        assert_eq!(input_get_u32(1002).unwrap(), 1234567);
        assert_eq!(input_get_u32(1004).unwrap(), 8901234);

        input_set_u32(900, 3412345).unwrap();
        assert_eq!(input_get_u32(900).unwrap(), 3412345);
    }

    #[test]
    fn test_get_input_set_f32() {
        let mut data_mem = alloc_stack!([f32; 2]);
        let mut data = FixedVec::new(&mut data_mem);

        data.push_all(&[1234.567, 890.1234]).unwrap();

        input_set_f32_bulk(2002, &data).unwrap();
        assert_eq!(input_get_f32(2002).unwrap(), 1234.567);
        assert_eq!(input_get_f32(2004).unwrap(), 890.1234);

        input_set_f32(2000, 1234.567).unwrap();
        assert_eq!(input_get_f32(2000).unwrap(), 1234.567);
    }

    #[test]
    fn test_get_bools_as_u8() {
        let mut data_mem = alloc_stack!([bool; CONTEXT_SIZE]);
        let mut data = FixedVec::new(&mut data_mem);
        coil_clear_all();
        data.push_all(&[true, true, true, true, true, true, false, false])
            .unwrap();
        coil_set_bulk(0, &data).unwrap();
        {
            with_context(&|context| {
                let mut result_mem = alloc_stack!([u8; CONTEXT_SIZE / 8]);
                let mut result = FixedVec::new(&mut result_mem);
                get_bools_as_u8(0, 6, &context.coils, &mut result).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b00111111);
                result.clear();
                get_bools_as_u8(0, 5, &context.coils, &mut result).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b00011111);
                result.clear();
            });
        }

        data.clear();
        data.push_all(&[true, true, false, true, true, true, true, true])
            .unwrap();
        coil_set_bulk(0, &data).unwrap();
        {
            with_context(&|context| {
                let mut result_mem = alloc_stack!([u8; CONTEXT_SIZE / 8]);
                let mut result = FixedVec::new(&mut result_mem);
                get_bools_as_u8(0, 6, &context.coils, &mut result).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b00111011);
                result.clear();
                get_bools_as_u8(0, 5, &context.coils, &mut result).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b00011011);
                result.clear();
            });
        }

        data.clear();
        data.push_all(&[
            true, true, false, true, true, true, true, true, // byte 1
            true, true, true, true, false, false, true, false, // byte 2
            false, false, false, true, false, true, // byte 3
        ])
        .unwrap();
        coil_set_bulk(0, &data).unwrap();
        {
            with_context(&|context| {
                let mut result_mem = alloc_stack!([u8; CONTEXT_SIZE / 8]);
                let mut result = FixedVec::new(&mut result_mem);
                get_bools_as_u8(0, 22, &context.coils, &mut result).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b11111011);
                assert_eq!(*result.get(1).unwrap(), 0b01001111);
                assert_eq!(*result.get(2).unwrap(), 0b101000);
            });
        }
    }

    #[test]
    fn test_get_set_regs_as_u8() {
        holding_clear_all();
        let mut data_mem = alloc_stack!([u16; CONTEXT_SIZE]);
        let mut data = FixedVec::new(&mut data_mem);
        data.push_all(&[2, 45, 4559, 31, 394, 1, 9, 7, 0, 1, 9])
            .unwrap();
        holding_set_bulk(0, &data).unwrap();
        with_mut_context(&|context| {
            let mut result_mem = alloc_stack!([u8; CONTEXT_SIZE]);
            let mut result = FixedVec::new(&mut result_mem);
            get_regs_as_u8(0, data.len() as u16, &context.holdings, &mut result).unwrap();
            set_regs_from_u8(0, &result, &mut context.holdings).unwrap();
        });
        let mut result_mem = alloc_stack!([u16; CONTEXT_SIZE]);
        let mut result = FixedVec::new(&mut result_mem);
        holding_get_bulk(0, data.len() as u16, &mut result).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_get_set_bools_as_u8() {
        coil_clear_all();
        let mut data_mem = alloc_stack!([bool; CONTEXT_SIZE]);
        let mut data = FixedVec::new(&mut data_mem);
        data.push_all(&[
            true, true, true, false, true, true, true, true, true, false, false, false, false,
            false,
        ])
        .unwrap();
        coil_set_bulk(0, &data).unwrap();
        coil_set(data.len() as u16, true).unwrap();
        coil_set(data.len() as u16 + 1, false).unwrap();
        coil_set(data.len() as u16 + 2, true).unwrap();
        with_mut_context(&|context| {
            let mut result_mem = alloc_stack!([u8; CONTEXT_SIZE]);
            let mut result = FixedVec::new(&mut result_mem);
            get_bools_as_u8(0, data.len() as u16, &context.coils, &mut result).unwrap();
            set_bools_from_u8(0, data.len() as u16, &result, &mut context.coils).unwrap();
        });
        let mut result_mem = alloc_stack!([bool; CONTEXT_SIZE]);
        let mut result = FixedVec::new(&mut result_mem);
        coil_get_bulk(0, data.len() as u16, &mut result).unwrap();
        assert_eq!(result, data);
        result.clear();
        data.push(true).unwrap();
        data.push(false).unwrap();
        data.push(true).unwrap();
        coil_get_bulk(0, data.len() as u16, &mut result).unwrap();
        assert_eq!(result, data);
    }

    /*
    //#[test]
    //fn test_dump_restore() {
    //let mut rng = rand::thread_rng();
    //let mut mycoils: Vec<bool> = Vec::new();
    //let mut mydiscretes: Vec<bool> = Vec::new();
    //let mut myholdings: Vec<u16> = Vec::new();
    //let mut myinputs: Vec<u16> = Vec::new();
    //for _ in 0..CONTEXT_SIZE {
    //mycoils.push(rng.gen());
    //mydiscretes.push(rng.gen());
    //myholdings.push(rng.gen());
    //myinputs.push(rng.gen());
    //}
    //clear_all();
    //coil_set_bulk(0, &mycoils).unwrap();
    //discrete_set_bulk(0, &mydiscretes).unwrap();
    //holding_set_bulk(0, &myholdings).unwrap();
    //input_set_bulk(0, &myinputs).unwrap();
    //let data = dump();
    //clear_all();
    //restore(&data).unwrap();
    //assert_eq!(coil_get_bulk(0, CONTEXT_SIZE as u16).unwrap(), mycoils);
    //assert_eq!(
    //discrete_get_bulk(0, CONTEXT_SIZE as u16).unwrap(),
    //mydiscretes
    //);
    //assert_eq!(
    //holding_get_bulk(0, CONTEXT_SIZE as u16).unwrap(),
    //myholdings
    //);
    //assert_eq!(input_get_bulk(0, CONTEXT_SIZE as u16).unwrap(), myinputs);
    //}*/
}
