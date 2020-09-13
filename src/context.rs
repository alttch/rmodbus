use ieee754::Ieee754;
use std::sync::{Mutex, MutexGuard};

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

/// Locks context read-only and calls a sub-function
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
    let ctx = CONTEXT.lock().unwrap();
    f(&ctx);
}

/// Locks context read-write and calls a sub-function
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
    let mut ctx = CONTEXT.lock().unwrap();
    f(&mut ctx);
}

//
// import / export
//

/// Dump full Modbus context to Vec<u8>
pub fn dump() -> Vec<u8> {
    let ctx = CONTEXT.lock().unwrap();
    return dump_locked(&ctx);
}

/// Dump full Modbus context when it's locked
pub fn dump_locked(context: &MutexGuard<ModbusContext>) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::new();
    data.append(&mut get_bools_as_u8(0, CONTEXT_SIZE as u16, &context.coils).unwrap());
    data.append(&mut get_bools_as_u8(0, CONTEXT_SIZE as u16, &context.discretes).unwrap());
    data.append(&mut get_regs_as_u8(0, CONTEXT_SIZE as u16, &context.holdings).unwrap());
    data.append(&mut get_regs_as_u8(0, CONTEXT_SIZE as u16, &context.inputs).unwrap());
    return data;
}

/// Restore full Modbus context to external binary file
pub fn restore(data: &Vec<u8>) -> Result<(), Error> {
    let mut ctx = CONTEXT.lock().unwrap();
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
        println!("wrong size {} {}", bool_size * 2 + reg_size * 2, data.len());
        return Err(Error {});
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
}

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
) -> Result<Vec<u8>, Error> {
    let reg_to = reg as usize + count as usize;
    if reg_to > CONTEXT_SIZE {
        return Err(Error {});
    }
    let mut result: Vec<u8> = Vec::new();
    for c in reg as usize..reg_to {
        result.push((reg_context[c] >> 8) as u8);
        result.push(reg_context[c] as u8);
    }
    return Ok(result);
}

/// Set 16-bit registers from &Vec<u8>
///
/// Useful for import / export and external API calls
pub fn set_regs_from_u8(
    reg: u16,
    values: &Vec<u8>,
    reg_context: &mut [u16; CONTEXT_SIZE],
) -> Result<(), Error> {
    if reg as usize + values.len() / 2 > CONTEXT_SIZE {
        return Err(Error {});
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
) -> Result<Vec<u8>, Error> {
    let reg_to = reg as usize + count as usize;
    if reg_to > CONTEXT_SIZE {
        return Err(Error {});
    }
    let mut result: Vec<u8> = Vec::new();
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
        result.push(cbyte);
    }
    return Ok(result);
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
    values: &Vec<u8>,
    reg_context: &mut [bool; CONTEXT_SIZE],
) -> Result<(), Error> {
    let reg_to = reg as usize + count as usize;
    if reg_to > CONTEXT_SIZE {
        return Err(Error {});
    }
    let mut creg = reg as usize;
    let mut cbyte = 0;
    let mut cnt = 0;
    while creg < reg_to && cnt < count {
        let mut b: u8 = match values.get(cbyte) {
            Some(v) => *v,
            None => return Err(Error {}),
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
/// Can get the coils (Vec<bool>) or 16-bit registers (Vec<u16>)
pub fn get_bulk<T: Copy>(
    reg: u16,
    count: u16,
    reg_context: &[T; CONTEXT_SIZE],
) -> Result<Vec<T>, Error> {
    let reg_to = reg as usize + count as usize;
    if reg_to > CONTEXT_SIZE {
        return Err(Error {});
    }
    let mut result: Vec<T> = Vec::new();
    result.extend_from_slice(&reg_context[reg as usize..reg_to]);
    return Ok(result);
}

/// Bulk set
///
/// Can set the coils from Vec<bool> or 16-bit registers from Vec<u16>
pub fn set_bulk<T: Copy>(
    reg: u16,
    data: &Vec<T>,
    reg_context: &mut [T; CONTEXT_SIZE],
) -> Result<(), Error> {
    if reg as usize + data.len() > CONTEXT_SIZE {
        return Err(Error {});
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
        return Err(Error {});
    }
    return Ok(reg_context[reg as usize]);
}

/// Set a single register
///
/// Set coil from bool or 16-bit reg from u16
pub fn set<T>(reg: u16, value: T, reg_context: &mut [T; CONTEXT_SIZE]) -> Result<(), Error> {
    if reg as usize >= CONTEXT_SIZE {
        return Err(Error {});
    }
    reg_context[reg as usize] = value;
    return Ok(());
}

/// Get two 16-bit registers as u32
///
/// Returns big-endian 32-bit value
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
/// Uses big-endian 32-bit value to set two registers
pub fn set_u32(reg: u16, value: u32, reg_context: &mut [u16; CONTEXT_SIZE]) -> Result<(), Error> {
    let mut data: Vec<u16> = Vec::new();
    data.push((value >> 16) as u16);
    data.push(value as u16);
    return set_bulk(reg, &data, reg_context);
}

/// Set multiple 16-bit registers from Vec<u32>
///
/// Uses big-endian 32-bit values to set the registers
pub fn set_u32_bulk(
    reg: u16,
    values: &Vec<u32>,
    reg_context: &mut [u16; CONTEXT_SIZE],
) -> Result<(), Error> {
    let mut data: Vec<u16> = Vec::new();
    for u in values {
        data.push((u >> 16) as u16);
        data.push(*u as u16);
    }
    return set_bulk(reg, &data, reg_context);
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
    values: &Vec<f32>,
    reg_context: &mut [u16; CONTEXT_SIZE],
) -> Result<(), Error> {
    let mut data: Vec<u32> = values.iter().map(|x| x.bits()).collect();
    for u in values {
        data.push(u.bits());
    }
    return set_u32_bulk(reg, &data, reg_context);
}

//
// coils functions
//

pub fn coil_get_bulk(reg: u16, count: u16) -> Result<Vec<bool>, Error> {
    let context = CONTEXT.lock().unwrap();
    return get_bulk(reg, count, &context.coils);
}

pub fn coil_set_bulk(reg: u16, coils: &Vec<bool>) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_bulk(reg, coils, &mut context.coils);
}

pub fn coil_get(reg: u16) -> Result<bool, Error> {
    let context = CONTEXT.lock().unwrap();
    return get(reg, &context.coils);
}

pub fn coil_set(reg: u16, value: bool) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set(reg, value, &mut context.coils);
}

//
// discretes functions
//

pub fn discrete_get_bulk(reg: u16, count: u16) -> Result<Vec<bool>, Error> {
    let context = CONTEXT.lock().unwrap();
    return get_bulk(reg, count, &context.discretes);
}

pub fn discrete_set_bulk(reg: u16, discretes: &Vec<bool>) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_bulk(reg, discretes, &mut context.discretes);
}

pub fn discrete_get(reg: u16) -> Result<bool, Error> {
    let context = CONTEXT.lock().unwrap();
    return get(reg, &context.discretes);
}

pub fn discrete_set(reg: u16, value: bool) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set(reg, value, &mut context.discretes);
}

//
// holdings functions
//

pub fn holding_get_bulk(reg: u16, count: u16) -> Result<Vec<u16>, Error> {
    let context = CONTEXT.lock().unwrap();
    return get_bulk(reg, count, &context.holdings);
}

pub fn holding_get(reg: u16) -> Result<u16, Error> {
    let context = CONTEXT.lock().unwrap();
    return get(reg, &context.holdings);
}

pub fn holding_get_u32(reg: u16) -> Result<u32, Error> {
    let context = CONTEXT.lock().unwrap();
    return get_u32(reg, &context.holdings);
}

pub fn holding_get_f32(reg: u16) -> Result<f32, Error> {
    let context = CONTEXT.lock().unwrap();
    return get_f32(reg, &context.holdings);
}

pub fn holding_set_bulk(reg: u16, holdings: &Vec<u16>) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_bulk(reg, &holdings, &mut context.holdings);
}

pub fn holding_set(reg: u16, value: u16) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set(reg, value, &mut context.holdings);
}

pub fn holding_set_u32_bulk(reg: u16, values: &Vec<u32>) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_u32_bulk(reg, values, &mut context.holdings);
}

pub fn holding_set_u32(reg: u16, value: u32) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_u32(reg, value, &mut context.holdings);
}

pub fn holding_set_f32_bulk(reg: u16, values: &Vec<f32>) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_f32_bulk(reg, values, &mut context.holdings);
}

pub fn holding_set_f32(reg: u16, value: f32) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_f32(reg, value, &mut context.holdings);
}

pub fn input_get_bulk(reg: u16, count: u16) -> Result<Vec<u16>, Error> {
    let context = CONTEXT.lock().unwrap();
    return get_bulk(reg, count, &context.inputs);
}

//
// input functions
//

pub fn input_get(reg: u16) -> Result<u16, Error> {
    let context = CONTEXT.lock().unwrap();
    return get(reg, &context.inputs);
}

pub fn input_get_u32(reg: u16) -> Result<u32, Error> {
    let context = CONTEXT.lock().unwrap();
    return get_u32(reg, &context.inputs);
}

pub fn input_get_f32(reg: u16) -> Result<f32, Error> {
    let context = CONTEXT.lock().unwrap();
    return get_f32(reg, &context.inputs);
}

pub fn input_set_bulk(reg: u16, inputs: &Vec<u16>) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_bulk(reg, inputs, &mut context.inputs);
}

pub fn input_set(reg: u16, value: u16) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set(reg, value, &mut context.inputs);
}

pub fn input_set_u32_bulk(reg: u16, values: &Vec<u32>) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_u32_bulk(reg, values, &mut context.inputs);
}

pub fn input_set_u32(reg: u16, value: u32) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_u32(reg, value, &mut context.inputs);
}

pub fn input_set_f32_bulk(reg: u16, values: &Vec<f32>) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_f32_bulk(reg, values, &mut context.inputs);
}

pub fn input_set_f32(reg: u16, value: f32) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_f32(reg, value, &mut context.inputs);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use std::fs::File;
    use std::io::prelude::*;

    #[test]
    fn test_read_coils_as_bytes_oob() {
        match coil_get_bulk(0, CONTEXT_SIZE as u16 + 1) {
            Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
            Err(_) => assert!(true),
        }
        match coil_get_bulk(CONTEXT_SIZE as u16, 1) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        coil_get_bulk((CONTEXT_SIZE - 1) as u16, 1).unwrap();
        match coil_get_bulk(CONTEXT_SIZE as u16 - 1, 2) {
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
        coil_set_bulk(5, &(vec![true; 2])).unwrap();
        assert_eq!(coil_get_bulk(5, 2).unwrap()[0..2], [true; 2]);
        coil_set_bulk(25, &(vec![true; 18])).unwrap();
        assert_eq!(coil_get_bulk(25, 18).unwrap()[0..18], [true; 18]);
        coil_set(28, true).unwrap();
        assert_eq!(coil_get(28).unwrap(), true);
        coil_set(28, false).unwrap();
        assert_eq!(coil_get(28).unwrap(), false);
    }

    #[test]
    fn test_read_discretes_as_bytes_oob() {
        match discrete_get_bulk(0, CONTEXT_SIZE as u16 + 1) {
            Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
            Err(_) => assert!(true),
        }
        match discrete_get_bulk(CONTEXT_SIZE as u16, 1) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        discrete_get_bulk((CONTEXT_SIZE - 1) as u16, 1).unwrap();
        match discrete_get_bulk(CONTEXT_SIZE as u16 - 1, 2) {
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
        discrete_clear_all();
        discrete_set_bulk(5, &(vec![true; 2])).unwrap();
        assert_eq!(discrete_get_bulk(5, 2).unwrap()[0..2], [true; 2]);
        discrete_set_bulk(25, &(vec![true; 18])).unwrap();
        assert_eq!(discrete_get_bulk(25, 18).unwrap()[0..18], [true; 18]);
        discrete_set(28, true).unwrap();
        assert_eq!(discrete_get(28).unwrap(), true);
        discrete_set(28, false).unwrap();
        assert_eq!(discrete_get(28).unwrap(), false);
    }

    #[test]
    fn test_read_holdings_as_bytes_oob() {
        match holding_get_bulk(0, CONTEXT_SIZE as u16 + 1) {
            Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
            Err(_) => assert!(true),
        }
        match holding_get_bulk(CONTEXT_SIZE as u16, 1) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        holding_get_bulk((CONTEXT_SIZE - 1) as u16, 1).unwrap();
        match holding_get_bulk(CONTEXT_SIZE as u16 - 1, 2) {
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
    }

    #[test]
    fn test_get_holding_set_bulk() {
        holding_clear_all();
        holding_set_bulk(5, &(vec![0x77; 2])).unwrap();
        assert_eq!(holding_get_bulk(5, 2).unwrap()[0..2], [0x77; 2]);
        holding_set_bulk(25, &(vec![0x33; 18])).unwrap();
        assert_eq!(holding_get_bulk(25, 18).unwrap()[0..18], [0x33; 18]);
        holding_set(28, 99).unwrap();
        assert_eq!(holding_get(28).unwrap(), 99);
        holding_set(28, 95).unwrap();
        assert_eq!(holding_get(28).unwrap(), 95);
        holding_set_u32(1000, 1234567).unwrap();
        assert_eq!(holding_get_u32(1000).unwrap(), 1234567);
        holding_set_u32_bulk(1002, &(vec![1234567, 8901234])).unwrap();
        assert_eq!(holding_get_u32(1002).unwrap(), 1234567);
        assert_eq!(holding_get_u32(1004).unwrap(), 8901234);
        holding_set_f32(2000, 1234.567).unwrap();
        assert_eq!(holding_get_f32(2000).unwrap(), 1234.567);
        holding_set_f32_bulk(2002, &(vec![1234.567, 890.1234])).unwrap();
        assert_eq!(holding_get_f32(2002).unwrap(), 1234.567);
        assert_eq!(holding_get_f32(2004).unwrap(), 890.1234);
    }

    #[test]
    fn test_read_inputs_as_bytes_oob() {
        match input_get_bulk(0, CONTEXT_SIZE as u16 + 1) {
            Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
            Err(_) => assert!(true),
        }
        match input_get_bulk(CONTEXT_SIZE as u16, 1) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        input_get_bulk((CONTEXT_SIZE - 1) as u16, 1).unwrap();
        match input_get_bulk(CONTEXT_SIZE as u16 - 1, 2) {
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
    }

    #[test]
    fn test_input_get_set_bulk() {
        input_clear_all();
        input_set_bulk(5, &(vec![0x77; 2])).unwrap();
        assert_eq!(input_get_bulk(5, 2).unwrap()[0..2], [0x77; 2]);
        input_set_bulk(25, &(vec![0x33; 18])).unwrap();
        assert_eq!(input_get_bulk(25, 18).unwrap()[0..18], [0x33; 18]);
        input_set(28, 99).unwrap();
        assert_eq!(input_get(28).unwrap(), 99);
        input_set(28, 95).unwrap();
        assert_eq!(input_get(28).unwrap(), 95);
        input_set_u32(1000, 1234567).unwrap();
        assert_eq!(input_get_u32(1000).unwrap(), 1234567);
        input_set_u32_bulk(1002, &(vec![1234567, 8901234])).unwrap();
        assert_eq!(input_get_u32(1002).unwrap(), 1234567);
        assert_eq!(input_get_u32(1004).unwrap(), 8901234);
        input_set_f32(2000, 1234.567).unwrap();
        assert_eq!(input_get_f32(2000).unwrap(), 1234.567);
        input_set_f32_bulk(2002, &(vec![1234.567, 890.1234])).unwrap();
        assert_eq!(input_get_f32(2002).unwrap(), 1234.567);
        assert_eq!(input_get_f32(2004).unwrap(), 890.1234);
    }

    #[test]
    fn test_get_bools_as_u8() {
        coil_clear_all();
        coil_set_bulk(0, &(vec![true, true, true, true, true, true, false, false])).unwrap();
        {
            with_context(&|context| {
                let result = get_bools_as_u8(0, 6, &context.coils).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b00111111);
                let result = get_bools_as_u8(0, 5, &context.coils).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b00011111);
            });
        }
        coil_set_bulk(0, &(vec![true, true, false, true, true, true, true, true])).unwrap();
        {
            with_context(&|context| {
                let result = get_bools_as_u8(0, 6, &context.coils).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b00111011);
                let result = get_bools_as_u8(0, 5, &context.coils).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b00011011);
            });
        }
        coil_set_bulk(
            0,
            &(vec![
                true, true, false, true, true, true, true, true, // byte 1
                true, true, true, true, false, false, true, false, // byte 2
                false, false, false, true, false, true, // byte 3
            ]),
        )
        .unwrap();
        {
            with_context(&|context| {
                let result = get_bools_as_u8(0, 22, &context.coils).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b11111011);
                assert_eq!(*result.get(1).unwrap(), 0b01001111);
                assert_eq!(*result.get(2).unwrap(), 0b101000);
            });
        }
    }

    #[test]
    fn test_get_set_regs_as_u8() {
        holding_clear_all();
        let data = vec![2, 45, 4559, 31, 394, 1, 9, 7, 0, 1, 9];
        holding_set_bulk(0, &data).unwrap();
        with_mut_context(&|context| {
            let result = get_regs_as_u8(0, data.len() as u16, &context.holdings).unwrap();
            set_regs_from_u8(0, &result, &mut context.holdings).unwrap();
        });
        assert_eq!(holding_get_bulk(0, data.len() as u16).unwrap(), data);
    }

    #[test]
    fn test_get_set_bools_as_u8() {
        coil_clear_all();
        let mut data = vec![
            true, true, true, false, true, true, true, true, true, false, false, false, false,
            false,
        ];
        coil_set_bulk(0, &data).unwrap();
        coil_set(data.len() as u16, true).unwrap();
        coil_set(data.len() as u16 + 1, false).unwrap();
        coil_set(data.len() as u16 + 2, true).unwrap();
        with_mut_context(&|context| {
            let result = get_bools_as_u8(0, data.len() as u16, &context.coils).unwrap();
            set_bools_from_u8(0, data.len() as u16, &result, &mut context.coils).unwrap();
        });
        assert_eq!(coil_get_bulk(0, data.len() as u16).unwrap(), data);
        data.push(true);
        data.push(false);
        data.push(true);
        assert_eq!(coil_get_bulk(0, data.len() as u16).unwrap(), data);
    }

    fn save(fname: &str) {
        let mut file = File::create(fname).unwrap();
        file.write_all(&dump()).unwrap();
        file.sync_all().unwrap();
    }

    fn load(fname: &str) {
        let mut file = File::open(fname).unwrap();
        let mut data: Vec<u8> = Vec::new();
        file.read_to_end(&mut data).unwrap();
        restore(&data).unwrap();
    }

    #[test]
    fn test_load_save() {
        let mut rng = rand::thread_rng();
        let mut mycoils: Vec<bool> = Vec::new();
        let mut mydiscretes: Vec<bool> = Vec::new();
        let mut myholdings: Vec<u16> = Vec::new();
        let mut myinputs: Vec<u16> = Vec::new();
        for _ in 0..CONTEXT_SIZE {
            mycoils.push(rng.gen());
            mydiscretes.push(rng.gen());
            myholdings.push(rng.gen());
            myinputs.push(rng.gen());
        }
        clear_all();
        coil_set_bulk(0, &mycoils).unwrap();
        discrete_set_bulk(0, &mydiscretes).unwrap();
        holding_set_bulk(0, &myholdings).unwrap();
        input_set_bulk(0, &myinputs).unwrap();
        save(&"/tmp/modbus-memory.dat");
        clear_all();
        load(&"/tmp/modbus-memory.dat");
        assert_eq!(coil_get_bulk(0, CONTEXT_SIZE as u16).unwrap(), mycoils);
        assert_eq!(
            discrete_get_bulk(0, CONTEXT_SIZE as u16).unwrap(),
            mydiscretes
        );
        assert_eq!(
            holding_get_bulk(0, CONTEXT_SIZE as u16).unwrap(),
            myholdings
        );
        assert_eq!(input_get_bulk(0, CONTEXT_SIZE as u16).unwrap(), myinputs);
    }
}
