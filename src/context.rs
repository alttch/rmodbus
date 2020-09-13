use ieee754::Ieee754;
use std::fs::File;
use std::io::prelude::*;
use std::sync::{Mutex, MutexGuard};

const CONTEXT_SIZE: usize = 10000;

pub struct ModbusContext {
    pub coils: [bool; CONTEXT_SIZE],
    pub discretes: [bool; CONTEXT_SIZE],
    pub holdings: [u16; CONTEXT_SIZE],
    pub inputs: [u16; CONTEXT_SIZE],
}

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
    pub static ref CONTEXT: Mutex<ModbusContext> = Mutex::new(ModbusContext::new());
}

pub fn with_context(f: &dyn Fn(&MutexGuard<ModbusContext>)) {
    let ctx = CONTEXT.lock().unwrap();
    f(&ctx);
}

pub fn with_mut_context(f: &dyn Fn(&mut MutexGuard<ModbusContext>)) {
    let mut ctx = CONTEXT.lock().unwrap();
    f(&mut ctx);
}

pub fn save(fname: &str) -> Result<(), std::io::Error> {
    let ctx = CONTEXT.lock().unwrap();
    return save_locked(fname, &ctx);
}

pub fn load(fname: &str) -> Result<(), std::io::Error> {
    let mut ctx = CONTEXT.lock().unwrap();
    return load_locked(fname, &mut ctx);
}

pub fn save_locked(fname: &str, context: &MutexGuard<ModbusContext>) -> Result<(), std::io::Error> {
    let mut file = match File::create(fname) {
        Ok(v) => v,
        Err(v) => return Err(v),
    };
    let mut data: Vec<u8> = Vec::new();
    data.append(&mut get_bools_as_u8(0, CONTEXT_SIZE as u16, &context.coils).unwrap());
    data.append(&mut get_bools_as_u8(0, CONTEXT_SIZE as u16, &context.discretes).unwrap());
    data.append(&mut get_regs_as_u8(0, CONTEXT_SIZE as u16, &context.holdings).unwrap());
    data.append(&mut get_regs_as_u8(0, CONTEXT_SIZE as u16, &context.inputs).unwrap());
    let _ = match file.write_all(&data) {
        Ok(_) => {}
        Err(v) => return Err(v),
    };
    let _ = match file.sync_data() {
        Ok(_) => {}
        Err(v) => return Err(v),
    };
    return Ok(());
}

pub fn load_locked(
    fname: &str,
    context: &mut MutexGuard<ModbusContext>,
) -> Result<(), std::io::Error> {
    let mut file = match File::open(fname) {
        Ok(v) => v,
        Err(v) => return Err(v),
    };
    let mut ubuffer = [0; CONTEXT_SIZE * 2];
    let mut bbuffer = [0; CONTEXT_SIZE / 8];
    match file.read(&mut bbuffer) {
        Ok(_) => {}
        Err(v) => return Err(v),
    };
    let mut data = Vec::new();
    data.extend_from_slice(&bbuffer);
    match set_bools_from_u8(0, (bbuffer.len() as u16) << 3, &data, &mut context.coils) {
        Ok(_) => {}
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "context error",
            ))
        }
    }
    match file.read(&mut bbuffer) {
        Ok(_) => {}
        Err(v) => return Err(v),
    };
    data.clear();
    data.extend_from_slice(&bbuffer);
    match set_bools_from_u8(
        0,
        (bbuffer.len() as u16) << 3,
        &data,
        &mut context.discretes,
    ) {
        Ok(_) => {}
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "context error",
            ))
        }
    }
    match file.read(&mut ubuffer) {
        Ok(_) => {}
        Err(v) => return Err(v),
    };
    data.clear();
    data.extend_from_slice(&ubuffer);
    match set_regs_from_u8(0, &data, &mut context.holdings) {
        Ok(_) => {}
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "context error",
            ))
        }
    }
    match file.read(&mut ubuffer) {
        Ok(_) => {}
        Err(v) => return Err(v),
    };
    data.clear();
    data.extend_from_slice(&ubuffer);
    match set_regs_from_u8(0, &data, &mut context.inputs) {
        Ok(_) => {}
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "context error",
            ))
        }
    }
    return Ok(());
}

pub fn clear_coils() {
    with_mut_context(&|context| {
        for i in 0..CONTEXT_SIZE {
            context.coils[i] = false;
        }
    });
}

pub fn clear_discretes() {
    with_mut_context(&|context| {
        for i in 0..CONTEXT_SIZE {
            context.discretes[i] = false;
        }
    });
}

pub fn clear_holdings() {
    with_mut_context(&|context| {
        for i in 0..CONTEXT_SIZE {
            context.holdings[i] = 0;
        }
    });
}

pub fn clear_inputs() {
    with_mut_context(&|context| {
        for i in 0..CONTEXT_SIZE {
            context.inputs[i] = 0;
        }
    });
}

pub fn clear() {
    with_mut_context(&|context| {
        for i in 0..CONTEXT_SIZE {
            context.coils[i] = false;
            context.discretes[i] = false;
            context.holdings[i] = 0;
            context.inputs[i] = 0;
        }
    });
}

pub fn get_regs_as_u8(
    reg: u16,
    count: u16,
    context: &[u16; CONTEXT_SIZE],
) -> Result<Vec<u8>, Error> {
    let reg_to = reg as usize + count as usize;
    if reg_to > CONTEXT_SIZE {
        return Err(Error {});
    }
    let mut result: Vec<u8> = Vec::new();
    for c in reg as usize..reg_to {
        result.push((context[c] >> 8) as u8);
        result.push(context[c] as u8);
    }
    return Ok(result);
}

pub fn set_regs_from_u8(
    reg: u16,
    values: &Vec<u8>,
    context: &mut [u16; CONTEXT_SIZE],
) -> Result<(), Error> {
    if reg as usize + values.len() / 2 > CONTEXT_SIZE {
        return Err(Error {});
    }
    let mut i = 0;
    let mut creg = reg as usize;
    while i < values.len() {
        context[creg] = u16::from_be_bytes([
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

pub fn get_bools_as_u8(
    reg: u16,
    count: u16,
    context: &[bool; CONTEXT_SIZE],
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
            if context[creg] {
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

pub fn set_bools_from_u8(
    reg: u16,
    count: u16,
    values: &Vec<u8>,
    context: &mut [bool; CONTEXT_SIZE],
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
            context[creg] = b & 1 == 1;
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

pub fn get_bulk<T: Copy>(
    reg: u16,
    count: u16,
    context: &[T; CONTEXT_SIZE],
) -> Result<Vec<T>, Error> {
    let reg_to = reg as usize + count as usize;
    if reg_to > CONTEXT_SIZE {
        return Err(Error {});
    }
    let mut result: Vec<T> = Vec::new();
    result.extend_from_slice(&context[reg as usize..reg_to]);
    return Ok(result);
}

pub fn set_bulk<T: Copy>(
    reg: u16,
    data: &Vec<T>,
    context: &mut [T; CONTEXT_SIZE],
) -> Result<(), Error> {
    if reg as usize + data.len() > CONTEXT_SIZE {
        return Err(Error {});
    }
    for (i, value) in data.iter().enumerate() {
        context[reg as usize + i] = *value;
    }
    return Ok(());
}

pub fn get<T: Copy>(reg: u16, context: &[T; CONTEXT_SIZE]) -> Result<T, Error> {
    if reg as usize >= CONTEXT_SIZE {
        return Err(Error {});
    }
    return Ok(context[reg as usize]);
}

pub fn set<T>(reg: u16, value: T, context: &mut [T; CONTEXT_SIZE]) -> Result<(), Error> {
    if reg as usize >= CONTEXT_SIZE {
        return Err(Error {});
    }
    context[reg as usize] = value;
    return Ok(());
}

pub fn get_u32(reg: u16, context: &[u16; CONTEXT_SIZE]) -> Result<u32, Error> {
    let w1 = match get(reg, context) {
        Ok(v) => v,
        Err(v) => return Err(v),
    };
    let w2 = match get(reg + 1, context) {
        Ok(v) => v,
        Err(v) => return Err(v),
    };
    return Ok(((w1 as u32) << 16) + w2 as u32);
}

pub fn set_u32(reg: u16, value: u32, context: &mut [u16; CONTEXT_SIZE]) -> Result<(), Error> {
    let mut data: Vec<u16> = Vec::new();
    data.push((value >> 16) as u16);
    data.push(value as u16);
    return set_bulk(reg, &data, context);
}

pub fn set_u32_bulk(
    reg: u16,
    values: &Vec<u32>,
    context: &mut [u16; CONTEXT_SIZE],
) -> Result<(), Error> {
    let mut data: Vec<u16> = Vec::new();
    for u in values {
        data.push((u >> 16) as u16);
        data.push(*u as u16);
    }
    return set_bulk(reg, &data, context);
}

pub fn get_f32(reg: u16, context: &[u16; CONTEXT_SIZE]) -> Result<f32, Error> {
    let i = match get_u32(reg, context) {
        Ok(v) => v,
        Err(v) => return Err(v),
    };
    return Ok(Ieee754::from_bits(i));
}

pub fn set_f32(reg: u16, value: f32, context: &mut [u16; CONTEXT_SIZE]) -> Result<(), Error> {
    return set_u32(reg, value.bits(), context);
}

pub fn set_f32_bulk(
    reg: u16,
    values: &Vec<f32>,
    context: &mut [u16; CONTEXT_SIZE],
) -> Result<(), Error> {
    let mut data: Vec<u32> = values.iter().map(|x| x.bits()).collect();
    for u in values {
        data.push(u.bits());
    }
    return set_u32_bulk(reg, &data, context);
}

pub fn get_coils(reg: u16, count: u16) -> Result<Vec<bool>, Error> {
    let context = CONTEXT.lock().unwrap();
    return get_bulk(reg, count, &context.coils);
}

pub fn set_coils(reg: u16, coils: &Vec<bool>) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_bulk(reg, coils, &mut context.coils);
}

pub fn get_coil(reg: u16) -> Result<bool, Error> {
    let context = CONTEXT.lock().unwrap();
    return get(reg, &context.coils);
}

pub fn set_coil(reg: u16, value: bool) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set(reg, value, &mut context.coils);
}

pub fn get_discretes(reg: u16, count: u16) -> Result<Vec<bool>, Error> {
    let context = CONTEXT.lock().unwrap();
    return get_bulk(reg, count, &context.discretes);
}

pub fn set_discretes(reg: u16, discretes: &Vec<bool>) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_bulk(reg, discretes, &mut context.discretes);
}

pub fn get_discrete(reg: u16) -> Result<bool, Error> {
    let context = CONTEXT.lock().unwrap();
    return get(reg, &context.discretes);
}

pub fn set_discrete(reg: u16, value: bool) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set(reg, value, &mut context.discretes);
}

pub fn get_holdings(reg: u16, count: u16) -> Result<Vec<u16>, Error> {
    let context = CONTEXT.lock().unwrap();
    return get_bulk(reg, count, &context.holdings);
}

pub fn get_holding(reg: u16) -> Result<u16, Error> {
    let context = CONTEXT.lock().unwrap();
    return get(reg, &context.holdings);
}

pub fn get_holding_u32(reg: u16) -> Result<u32, Error> {
    let context = CONTEXT.lock().unwrap();
    return get_u32(reg, &context.holdings);
}

pub fn get_holding_f32(reg: u16) -> Result<f32, Error> {
    let context = CONTEXT.lock().unwrap();
    return get_f32(reg, &context.holdings);
}

pub fn set_holdings(reg: u16, holdings: &Vec<u16>) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_bulk(reg, &holdings, &mut context.holdings);
}

pub fn set_holding(reg: u16, value: u16) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set(reg, value, &mut context.holdings);
}

pub fn set_holdings_u32_bulk(reg: u16, values: &Vec<u32>) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_u32_bulk(reg, values, &mut context.holdings);
}

pub fn set_holding_u32(reg: u16, value: u32) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_u32(reg, value, &mut context.holdings);
}

pub fn set_holdings_f32_bulk(reg: u16, values: &Vec<f32>) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_f32_bulk(reg, values, &mut context.holdings);
}

pub fn set_holding_f32(reg: u16, value: f32) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_f32(reg, value, &mut context.holdings);
}

pub fn get_inputs(reg: u16, count: u16) -> Result<Vec<u16>, Error> {
    let context = CONTEXT.lock().unwrap();
    return get_bulk(reg, count, &context.inputs);
}

pub fn get_input(reg: u16) -> Result<u16, Error> {
    let context = CONTEXT.lock().unwrap();
    return get(reg, &context.inputs);
}

pub fn get_input_u32(reg: u16) -> Result<u32, Error> {
    let context = CONTEXT.lock().unwrap();
    return get_u32(reg, &context.inputs);
}

pub fn get_input_f32(reg: u16) -> Result<f32, Error> {
    let context = CONTEXT.lock().unwrap();
    return get_f32(reg, &context.inputs);
}

pub fn set_inputs(reg: u16, inputs: &Vec<u16>) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_bulk(reg, inputs, &mut context.inputs);
}

pub fn set_input(reg: u16, value: u16) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set(reg, value, &mut context.inputs);
}

pub fn set_inputs_u32_bulk(reg: u16, values: &Vec<u32>) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_u32_bulk(reg, values, &mut context.inputs);
}

pub fn set_input_u32(reg: u16, value: u32) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_u32(reg, value, &mut context.inputs);
}

pub fn set_inputs_f32_bulk(reg: u16, values: &Vec<f32>) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_f32_bulk(reg, values, &mut context.inputs);
}

pub fn set_input_f32(reg: u16, value: f32) -> Result<(), Error> {
    let mut context = CONTEXT.lock().unwrap();
    return set_f32(reg, value, &mut context.inputs);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_coils_as_bytes_oob() {
        match get_coils(0, CONTEXT_SIZE as u16 + 1) {
            Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
            Err(_) => assert!(true),
        }
        match get_coils(CONTEXT_SIZE as u16, 1) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        get_coils((CONTEXT_SIZE - 1) as u16, 1).unwrap();
        match get_coils(CONTEXT_SIZE as u16 - 1, 2) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        get_coil((CONTEXT_SIZE - 1) as u16).unwrap();
        match get_coil(CONTEXT_SIZE as u16) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        set_coil((CONTEXT_SIZE - 1) as u16, true).unwrap();
        match set_coil(CONTEXT_SIZE as u16, true) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_get_set_coils() {
        set_coils(5, &(vec![true; 2])).unwrap();
        assert_eq!(get_coils(5, 2).unwrap()[0..2], [true; 2]);
        set_coils(25, &(vec![true; 18])).unwrap();
        assert_eq!(get_coils(25, 18).unwrap()[0..18], [true; 18]);
        set_coil(28, true).unwrap();
        assert_eq!(get_coil(28).unwrap(), true);
        set_coil(28, false).unwrap();
        assert_eq!(get_coil(28).unwrap(), false);
    }

    #[test]
    fn test_read_discretes_as_bytes_oob() {
        match get_discretes(0, CONTEXT_SIZE as u16 + 1) {
            Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
            Err(_) => assert!(true),
        }
        match get_discretes(CONTEXT_SIZE as u16, 1) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        get_discretes((CONTEXT_SIZE - 1) as u16, 1).unwrap();
        match get_discretes(CONTEXT_SIZE as u16 - 1, 2) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        get_discrete((CONTEXT_SIZE - 1) as u16).unwrap();
        match get_discrete(CONTEXT_SIZE as u16) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        set_discrete((CONTEXT_SIZE - 1) as u16, true).unwrap();
        match set_discrete(CONTEXT_SIZE as u16, true) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_get_set_discretes() {
        clear_discretes();
        set_discretes(5, &(vec![true; 2])).unwrap();
        assert_eq!(get_discretes(5, 2).unwrap()[0..2], [true; 2]);
        set_discretes(25, &(vec![true; 18])).unwrap();
        assert_eq!(get_discretes(25, 18).unwrap()[0..18], [true; 18]);
        set_discrete(28, true).unwrap();
        assert_eq!(get_discrete(28).unwrap(), true);
        set_discrete(28, false).unwrap();
        assert_eq!(get_discrete(28).unwrap(), false);
    }

    #[test]
    fn test_read_holdings_as_bytes_oob() {
        match get_holdings(0, CONTEXT_SIZE as u16 + 1) {
            Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
            Err(_) => assert!(true),
        }
        match get_holdings(CONTEXT_SIZE as u16, 1) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        get_holdings((CONTEXT_SIZE - 1) as u16, 1).unwrap();
        match get_holdings(CONTEXT_SIZE as u16 - 1, 2) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        get_holding((CONTEXT_SIZE - 1) as u16).unwrap();
        match get_holding(CONTEXT_SIZE as u16) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        set_holding((CONTEXT_SIZE - 1) as u16, 0x55).unwrap();
        match set_holding(CONTEXT_SIZE as u16, 0x55) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        match set_holding_u32((CONTEXT_SIZE - 1) as u16, 0x55) {
            Ok(_) => assert!(false, "oob failed MAX u32"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_get_set_holdings() {
        clear_holdings();
        set_holdings(5, &(vec![0x77; 2])).unwrap();
        assert_eq!(get_holdings(5, 2).unwrap()[0..2], [0x77; 2]);
        set_holdings(25, &(vec![0x33; 18])).unwrap();
        assert_eq!(get_holdings(25, 18).unwrap()[0..18], [0x33; 18]);
        set_holding(28, 99).unwrap();
        assert_eq!(get_holding(28).unwrap(), 99);
        set_holding(28, 95).unwrap();
        assert_eq!(get_holding(28).unwrap(), 95);
        set_holding_u32(1000, 1234567).unwrap();
        assert_eq!(get_holding_u32(1000).unwrap(), 1234567);
        set_holdings_u32_bulk(1002, &(vec![1234567, 8901234])).unwrap();
        assert_eq!(get_holding_u32(1002).unwrap(), 1234567);
        assert_eq!(get_holding_u32(1004).unwrap(), 8901234);
        set_holding_f32(2000, 1234.567).unwrap();
        assert_eq!(get_holding_f32(2000).unwrap(), 1234.567);
        set_holdings_f32_bulk(2002, &(vec![1234.567, 890.1234])).unwrap();
        assert_eq!(get_holding_f32(2002).unwrap(), 1234.567);
        assert_eq!(get_holding_f32(2004).unwrap(), 890.1234);
    }

    #[test]
    fn test_read_inputs_as_bytes_oob() {
        match get_inputs(0, CONTEXT_SIZE as u16 + 1) {
            Ok(_) => assert!(false, "oob failed 0 - MAX+1 "),
            Err(_) => assert!(true),
        }
        match get_inputs(CONTEXT_SIZE as u16, 1) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        get_inputs((CONTEXT_SIZE - 1) as u16, 1).unwrap();
        match get_inputs(CONTEXT_SIZE as u16 - 1, 2) {
            Ok(_) => assert!(false, "oob failed MAX - MAX+1"),
            Err(_) => assert!(true),
        }
        get_input((CONTEXT_SIZE - 1) as u16).unwrap();
        match get_input(CONTEXT_SIZE as u16) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        set_input((CONTEXT_SIZE - 1) as u16, 0x55).unwrap();
        match set_input(CONTEXT_SIZE as u16, 0x55) {
            Ok(_) => assert!(false, "oob failed MAX"),
            Err(_) => assert!(true),
        }
        match set_input_u32((CONTEXT_SIZE - 1) as u16, 0x55) {
            Ok(_) => assert!(false, "oob failed MAX u32"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_get_set_inputs() {
        clear_inputs();
        set_inputs(5, &(vec![0x77; 2])).unwrap();
        assert_eq!(get_inputs(5, 2).unwrap()[0..2], [0x77; 2]);
        set_inputs(25, &(vec![0x33; 18])).unwrap();
        assert_eq!(get_inputs(25, 18).unwrap()[0..18], [0x33; 18]);
        set_input(28, 99).unwrap();
        assert_eq!(get_input(28).unwrap(), 99);
        set_input(28, 95).unwrap();
        assert_eq!(get_input(28).unwrap(), 95);
        set_input_u32(1000, 1234567).unwrap();
        assert_eq!(get_input_u32(1000).unwrap(), 1234567);
        set_inputs_u32_bulk(1002, &(vec![1234567, 8901234])).unwrap();
        assert_eq!(get_input_u32(1002).unwrap(), 1234567);
        assert_eq!(get_input_u32(1004).unwrap(), 8901234);
        set_input_f32(2000, 1234.567).unwrap();
        assert_eq!(get_input_f32(2000).unwrap(), 1234.567);
        set_inputs_f32_bulk(2002, &(vec![1234.567, 890.1234])).unwrap();
        assert_eq!(get_input_f32(2002).unwrap(), 1234.567);
        assert_eq!(get_input_f32(2004).unwrap(), 890.1234);
    }

    #[test]
    fn test_get_bools_as_u8() {
        clear_coils();
        set_coils(0, &(vec![true, true, true, true, true, true, false, false])).unwrap();
        {
            with_context(&|context| {
                let result = get_bools_as_u8(0, 6, &context.coils).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b00111111);
                let result = get_bools_as_u8(0, 5, &context.coils).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b00011111);
            });
        }
        set_coils(0, &(vec![true, true, false, true, true, true, true, true])).unwrap();
        {
            with_context(&|context| {
                let result = get_bools_as_u8(0, 6, &context.coils).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b00111011);
                let result = get_bools_as_u8(0, 5, &context.coils).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b00011011);
            });
        }
        set_coils(
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
        clear_holdings();
        let data = vec![2, 45, 4559, 31, 394, 1, 9, 7, 0, 1, 9];
        set_holdings(0, &data).unwrap();
        with_mut_context(&|context| {
            let result = get_regs_as_u8(0, data.len() as u16, &context.holdings).unwrap();
            set_regs_from_u8(0, &result, &mut context.holdings).unwrap();
        });
        assert_eq!(get_holdings(0, data.len() as u16).unwrap(), data);
    }

    #[test]
    fn test_get_set_bools_as_u8() {
        clear_coils();
        let mut data = vec![
            true, true, true, false, true, true, true, true, true, false, false, false, false,
            false,
        ];
        set_coils(0, &data).unwrap();
        set_coil(data.len() as u16, true).unwrap();
        set_coil(data.len() as u16 + 1, false).unwrap();
        set_coil(data.len() as u16 + 2, true).unwrap();
        with_mut_context(&|context| {
            let result = get_bools_as_u8(0, data.len() as u16, &context.coils).unwrap();
            set_bools_from_u8(0, data.len() as u16, &result, &mut context.coils).unwrap();
        });
        assert_eq!(get_coils(0, data.len() as u16).unwrap(), data);
        data.push(true);
        data.push(false);
        data.push(true);
        assert_eq!(get_coils(0, data.len() as u16).unwrap(), data);
    }

    #[test]
    fn test_load_save() {
        use rand::Rng;
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
        clear();
        set_coils(0, &mycoils).unwrap();
        set_discretes(0, &mydiscretes).unwrap();
        set_holdings(0, &myholdings).unwrap();
        set_inputs(0, &myinputs).unwrap();
        save(&"/tmp/modbus-memory.dat").unwrap();
        clear();
        load(&"/tmp/modbus-memory.dat").unwrap();
        assert_eq!(get_coils(0, CONTEXT_SIZE as u16).unwrap(), mycoils);
        assert_eq!(get_discretes(0, CONTEXT_SIZE as u16).unwrap(), mydiscretes);
        assert_eq!(get_holdings(0, CONTEXT_SIZE as u16).unwrap(), myholdings);
        assert_eq!(get_inputs(0, CONTEXT_SIZE as u16).unwrap(), myinputs);
    }
}
