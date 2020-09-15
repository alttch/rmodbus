use std::sync::{Mutex, MutexGuard};

impl<T: Copy> VectorTrait<T> for Vec<T> {
    fn add(&mut self, value: T) -> Result<(), ErrorKind> {
        self.push(value);
        return Ok(());
    }
    fn add_bulk(&mut self, values: &[T]) -> Result<(), ErrorKind> {
        self.extend_from_slice(values);
        return Ok(());
    }
    fn get_len(&self) -> usize {
        return self.len();
    }
    fn clear_all(&mut self) {
        self.clear();
    }
    fn cut_end(&mut self, len_to_cut: usize, value: T) {
        let len = self.len();
        if len_to_cut >= len {
            self.clear();
        } else {
            self.resize(len - len_to_cut, value);
        }
    }
    fn get_slice(&self) -> &[T] {
        return self.as_slice();
    }
}

macro_rules! lock_mutex {
    ($v:path) => {
        $v.lock().unwrap()
    };
}

include!("rmodbus.rs");

#[cfg(test)]
mod tests {
    use super::server::context::*;
    use rand::Rng;

    #[test]
    fn test_std_read_coils_as_bytes_oob() {
        let mut result = Vec::new();
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
    fn test_std_coil_get_set_bulk() {
        let mut data = Vec::new();
        let mut result = Vec::new();
        data.extend_from_slice(&[true; 2]);
        coil_set_bulk(5, &data.as_slice()).unwrap();
        coil_get_bulk(5, 2, &mut result).unwrap();
        assert_eq!(result, data);

        data.clear();
        result.clear();

        data.extend_from_slice(&[true; 18]);
        coil_set_bulk(25, &data.as_slice()).unwrap();
        coil_get_bulk(25, 18, &mut result).unwrap();
        assert_eq!(result, data);

        coil_set(28, true).unwrap();
        assert_eq!(coil_get(28).unwrap(), true);
        coil_set(28, false).unwrap();
        assert_eq!(coil_get(28).unwrap(), false);
    }

    #[test]
    fn test_std_read_discretes_as_bytes_oob() {
        let mut result = Vec::new();
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
    fn test_std_discrete_get_set_bulk() {
        let mut data = Vec::new();
        let mut result = Vec::new();
        data.extend_from_slice(&[true; 2]);
        discrete_set_bulk(5, &data.as_slice()).unwrap();
        discrete_get_bulk(5, 2, &mut result).unwrap();
        assert_eq!(result, data);

        data.clear();
        result.clear();

        data.extend_from_slice(&[true; 18]);
        discrete_set_bulk(25, &data.as_slice()).unwrap();
        discrete_get_bulk(25, 18, &mut result).unwrap();
        assert_eq!(result, data);

        discrete_set(28, true).unwrap();
        assert_eq!(discrete_get(28).unwrap(), true);
        discrete_set(28, false).unwrap();
        assert_eq!(discrete_get(28).unwrap(), false);
    }

    #[test]
    fn test_std_read_holdings_as_bytes_oob() {
        let mut result = Vec::new();
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
    fn test_std_get_holding_set_bulk() {
        let mut data = Vec::new();
        let mut result = Vec::new();

        holding_clear_all();

        data.extend_from_slice(&[0x77; 2]);
        holding_set_bulk(5, &data.as_slice()).unwrap();
        holding_get_bulk(5, 2, &mut result).unwrap();
        assert_eq!(result, data);

        data.clear();
        result.clear();

        data.extend_from_slice(&[0x33; 18]);
        holding_set_bulk(25, &data.as_slice()).unwrap();
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
    fn test_std_get_holding_set_u32() {
        let mut data = Vec::new();

        data.extend_from_slice(&[1234567, 8901234]);
        holding_set_u32_bulk(1002, &data.as_slice()).unwrap();

        assert_eq!(holding_get_u32(1002).unwrap(), 1234567);
        assert_eq!(holding_get_u32(1004).unwrap(), 8901234);

        holding_set_u32(900, 3412345).unwrap();
        assert_eq!(holding_get_u32(900).unwrap(), 3412345);
    }

    #[test]
    fn test_std_get_holding_set_f32() {
        let mut data = Vec::new();

        data.extend_from_slice(&[1234.567, 890.1234]);

        holding_set_f32_bulk(2002, &data.as_slice()).unwrap();
        assert_eq!(holding_get_f32(2002).unwrap(), 1234.567);
        assert_eq!(holding_get_f32(2004).unwrap(), 890.1234);

        holding_set_f32(2000, 1234.567).unwrap();
        assert_eq!(holding_get_f32(2000).unwrap(), 1234.567);
    }

    #[test]
    fn test_std_read_inputs_as_bytes_oob() {
        let mut result = Vec::new();
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
    fn test_std_get_input_set_bulk() {
        let mut data = Vec::new();
        let mut result = Vec::new();

        input_clear_all();

        data.extend_from_slice(&[0x77; 2]);
        input_set_bulk(5, &data.as_slice()).unwrap();
        input_get_bulk(5, 2, &mut result).unwrap();
        assert_eq!(result, data);

        data.clear();
        result.clear();

        data.extend_from_slice(&[0x33; 18]);
        input_set_bulk(25, &data.as_slice()).unwrap();
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
    fn test_std_get_input_set_u32() {
        let mut data = Vec::new();

        data.extend_from_slice(&[1234567, 8901234]);
        input_set_u32_bulk(1002, &data.as_slice()).unwrap();

        assert_eq!(input_get_u32(1002).unwrap(), 1234567);
        assert_eq!(input_get_u32(1004).unwrap(), 8901234);

        input_set_u32(900, 3412345).unwrap();
        assert_eq!(input_get_u32(900).unwrap(), 3412345);
    }

    #[test]
    fn test_std_get_input_set_f32() {
        let mut data = Vec::new();

        data.extend_from_slice(&[1234.567, 890.1234]);

        input_set_f32_bulk(2002, &data.as_slice()).unwrap();
        assert_eq!(input_get_f32(2002).unwrap(), 1234.567);
        assert_eq!(input_get_f32(2004).unwrap(), 890.1234);

        input_set_f32(2000, 1234.567).unwrap();
        assert_eq!(input_get_f32(2000).unwrap(), 1234.567);
    }

    #[test]
    fn test_std_get_bools_as_u8() {
        let mut data = Vec::new();
        coil_clear_all();
        data.extend_from_slice(&[true, true, true, true, true, true, false, false]);
        coil_set_bulk(0, &data.as_slice()).unwrap();
        {
            with_context(&|context| {
                let mut result = Vec::new();
                get_bools_as_u8(0, 6, &context.coils, &mut result).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b00111111);
                result.clear();
                get_bools_as_u8(0, 5, &context.coils, &mut result).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b00011111);
                result.clear();
            });
        }

        data.clear();
        data.extend_from_slice(&[true, true, false, true, true, true, true, true]);
        coil_set_bulk(0, &data.as_slice()).unwrap();
        {
            with_context(&|context| {
                let mut result = Vec::new();
                get_bools_as_u8(0, 6, &context.coils, &mut result).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b00111011);
                result.clear();
                get_bools_as_u8(0, 5, &context.coils, &mut result).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b00011011);
                result.clear();
            });
        }

        data.clear();
        data.extend_from_slice(&[
            true, true, false, true, true, true, true, true, // byte 1
            true, true, true, true, false, false, true, false, // byte 2
            false, false, false, true, false, true, // byte 3
        ]);
        coil_set_bulk(0, &data.as_slice()).unwrap();
        {
            with_context(&|context| {
                let mut result = Vec::new();
                get_bools_as_u8(0, 22, &context.coils, &mut result).unwrap();
                assert_eq!(*result.get(0).unwrap(), 0b11111011);
                assert_eq!(*result.get(1).unwrap(), 0b01001111);
                assert_eq!(*result.get(2).unwrap(), 0b101000);
            });
        }
    }

    #[test]
    fn test_std_get_set_regs_as_u8() {
        holding_clear_all();
        let mut data = Vec::new();
        data.extend_from_slice(&[2, 45, 4559, 31, 394, 1, 9, 7, 0, 1, 9]);
        holding_set_bulk(0, &data.as_slice()).unwrap();
        with_mut_context(&|context| {
            let mut result = Vec::new();
            get_regs_as_u8(0, data.len() as u16, &context.holdings, &mut result).unwrap();
            assert_eq!(result[0], 0);
            assert_eq!(result[1], 2);
            for i in 0..10 {
                set(i, 0, &mut context.holdings).unwrap();
            }
            set_regs_from_u8(0, &result.as_slice(), &mut context.holdings).unwrap();
        });
        let mut result = Vec::new();
        holding_get_bulk(0, data.len() as u16, &mut result).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_std_get_set_bools_as_u8() {
        coil_clear_all();
        let mut data = Vec::new();
        data.extend_from_slice(&[
            true, true, true, false, true, true, true, true, true, false, false, false, false,
            false,
        ]);
        coil_set_bulk(0, &data.as_slice()).unwrap();
        coil_set(data.len() as u16, true).unwrap();
        coil_set(data.len() as u16 + 1, false).unwrap();
        coil_set(data.len() as u16 + 2, true).unwrap();
        with_mut_context(&|context| {
            let mut result = Vec::new();
            get_bools_as_u8(0, data.len() as u16, &context.coils, &mut result).unwrap();
            set_bools_from_u8(0, data.len() as u16, &result.as_slice(), &mut context.coils)
                .unwrap();
        });
        let mut result = Vec::new();
        coil_get_bulk(0, data.len() as u16, &mut result).unwrap();
        assert_eq!(result, data);
        result.clear();
        data.push(true);
        data.push(false);
        data.push(true);
        coil_get_bulk(0, data.len() as u16, &mut result).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_std_dump_restore() {
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
        let mut dump: Vec<u8> = Vec::new();
        {
            let ctx = lock_mutex!(CONTEXT);
            for i in 0..CONTEXT_SIZE * 17 / 4 {
                dump.push(get_context_cell(i as u16, &ctx).unwrap());
            }
        }
        clear_all();
        let mut ctx = lock_mutex!(CONTEXT);
        let mut offset = 0;
        for value in &dump {
            set_context_cell(offset, *value, &mut ctx).unwrap();
            offset = offset + 1;
        }
        let mut result = Vec::new();
        get_bulk(0, CONTEXT_SIZE as u16, &ctx.coils, &mut result).unwrap();
        assert_eq!(result, mycoils);
        result.clear();
        get_bulk(0, CONTEXT_SIZE as u16, &ctx.discretes, &mut result).unwrap();
        assert_eq!(result, mydiscretes);
        result.clear();

        let mut result = Vec::new();
        get_bulk(0, CONTEXT_SIZE as u16, &ctx.inputs, &mut result).unwrap();
        assert_eq!(result, myinputs);
        result.clear();
        get_bulk(0, CONTEXT_SIZE as u16, &ctx.holdings, &mut result).unwrap();
        assert_eq!(result, myholdings);
        result.clear();

        let mut dump2: Vec<u8> = Vec::new();
        for value in context_iter(&ctx) {
            dump2.push(value);
        }
        assert_eq!(dump, dump2);
    }
}
