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

#[cfg(feature = "single")]
macro_rules! lock_mutex {
    ($v:path) => {
        $v.lock()
    };
}

#[cfg(not(feature = "single"))]
macro_rules! lock_mutex {
    ($v:path) => {
        $v.lock().unwrap()
    };
}

include!("rmodbus.rs");

#[cfg(test)]
mod tests {
    use super::server::context::*;
    use super::server::*;
    use super::ErrorKind;

    use crc16::*;
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
        holding_set_u32(100, 1234567).unwrap();
        assert_eq!(holding_get_u32(100).unwrap(), 1234567);
    }

    #[test]
    fn test_std_get_holding_set_u32() {
        let mut data = Vec::new();

        data.extend_from_slice(&[1234567, 8901234]);
        holding_set_u32_bulk(102, &data.as_slice()).unwrap();

        assert_eq!(holding_get_u32(102).unwrap(), 1234567);
        assert_eq!(holding_get_u32(104).unwrap(), 8901234);

        holding_set_u32(90, 3412345).unwrap();
        assert_eq!(holding_get_u32(90).unwrap(), 3412345);
    }

    #[test]
    fn test_std_get_holding_set_f32() {
        let mut data = Vec::new();

        data.extend_from_slice(&[1234.567, 890.1234]);

        holding_set_f32_bulk(202, &data.as_slice()).unwrap();
        assert_eq!(holding_get_f32(202).unwrap(), 1234.567);
        assert_eq!(holding_get_f32(204).unwrap(), 890.1234);

        holding_set_f32(200, 1234.567).unwrap();
        assert_eq!(holding_get_f32(200).unwrap(), 1234.567);
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
        input_set_u32(30, 1234567).unwrap();
        assert_eq!(input_get_u32(30).unwrap(), 1234567);
    }

    #[test]
    fn test_std_get_input_set_u32() {
        let mut data = Vec::new();

        data.extend_from_slice(&[1234567, 8901234]);
        input_set_u32_bulk(102, &data.as_slice()).unwrap();

        assert_eq!(input_get_u32(102).unwrap(), 1234567);
        assert_eq!(input_get_u32(104).unwrap(), 8901234);

        input_set_u32(50, 3412345).unwrap();
        assert_eq!(input_get_u32(50).unwrap(), 3412345);
    }

    #[test]
    fn test_std_get_input_set_f32() {
        let mut data = Vec::new();

        data.extend_from_slice(&[1234.567, 890.1234]);

        input_set_f32_bulk(202, &data.as_slice()).unwrap();
        assert_eq!(input_get_f32(202).unwrap(), 1234.567);
        assert_eq!(input_get_f32(204).unwrap(), 890.1234);

        input_set_f32(200, 1234.567).unwrap();
        assert_eq!(input_get_f32(200).unwrap(), 1234.567);
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

        drop(ctx);
        clear_all();

        let mut ctx = lock_mutex!(CONTEXT);
        let mut writer = ModbusContextWriter::new(0);
        for data in dump.chunks(500) {
            writer.write_bulk(&data, &mut ctx).unwrap();
        }

        let mut dump2: Vec<u8> = Vec::new();
        for value in context_iter(&ctx) {
            dump2.push(value);
        }

        assert_eq!(dump, dump2);
    }

    fn gen_tcp_frame(data: &[u8]) -> ModbusFrame {
        let mut frame: ModbusFrame = [0; 256];
        frame[0] = 0x77;
        frame[1] = 0x55;
        frame[2] = 0;
        frame[3] = 0;
        let len = (data.len() as u16).to_be_bytes();
        frame[4] = len[0];
        frame[5] = len[1];
        for (i, v) in data.iter().enumerate() {
            frame[i + 6] = *v;
        }
        return frame;
    }

    fn gen_rtu_frame(data: &[u8]) -> ModbusFrame {
        let mut frame: ModbusFrame = [0; 256];
        for (i, v) in data.iter().enumerate() {
            frame[i] = *v;
        }
        let len = data.len();
        let crc16 = State::<MODBUS>::calculate(data);
        let c = crc16.to_le_bytes();
        frame[len] = c[0];
        frame[len + 1] = c[1];
        return frame;
    }

    fn check_rtu_response(result: &Vec<u8>, response: &[u8]) {
        let mut resp = Vec::new();
        let mut r = Vec::new();
        for i in 6..response.len() {
            resp.push(response[i]);
        }
        for i in 0..result.len() - 2 {
            r.push(result[i]);
        }
        assert_eq!(resp, r);
        resp.insert(0, 1);
        let result_crc = u16::from_le_bytes([result[result.len() - 2], result[result.len() - 1]]);
        assert_eq!(result_crc, State::<MODBUS>::calculate(r.as_slice()));
    }

    #[test]
    fn test_std_frame_fc01_fc02_fc03_fc04_unknown_function() {
        clear_all();
        let mut result = Vec::new();
        // read coils
        coil_set(5, true).unwrap();
        coil_set(7, true).unwrap();
        coil_set(9, true).unwrap();
        let request = [1, 1, 0, 5, 0, 5];
        let response = [0x77, 0x55, 0, 0, 0, 4, 1, 1, 1, 0x15];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        let mut frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // check rtu crc error
        frame[request.len() + 1] = ((frame[request.len() + 1] as u16) + 1) as u8;
        match process_frame(1, &frame, ModbusProto::Rtu, &mut result) {
            Ok(_) => panic!(),
            Err(e) => match e {
                ErrorKind::FrameCRCError => {}
                _ => panic!(),
            },
        }
        // check illegal_function
        let request = [1, 7, 0x27, 0xe, 0, 0xf];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x87, 1];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // check context oob
        let request = [1, 1, 0x27, 0xe, 0, 0xf];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x81, 2];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // check invalid length
        let request = [1, 1, 0, 5, 0, 5];
        let mut frame = gen_tcp_frame(&request);
        frame[5] = 2;
        match process_frame(1, &frame, ModbusProto::TcpUdp, &mut result) {
            Ok(_) => panic!(),
            Err(e) => match e {
                ErrorKind::FrameBroken => {}
                _ => panic!("{:?}", e),
            },
        }
        let mut frame = gen_tcp_frame(&request);
        frame[5] = 251;
        match process_frame(1, &frame, ModbusProto::TcpUdp, &mut result) {
            Ok(_) => panic!(),
            Err(e) => match e {
                ErrorKind::FrameBroken => {}
                _ => panic!("{:?}", e),
            },
        }
        let mut frame = gen_tcp_frame(&request);
        frame[3] = 22;
        match process_frame(1, &frame, ModbusProto::TcpUdp, &mut result) {
            Ok(_) => panic!(),
            Err(e) => match e {
                ErrorKind::FrameBroken => {}
                _ => panic!("{:?}", e),
            },
        }
        // read discretes
        discrete_set(10, true).unwrap();
        discrete_set(12, true).unwrap();
        discrete_set(16, true).unwrap();
        let frame = gen_tcp_frame(&[1, 2, 0, 5, 0, 0x10]);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(
            result.as_slice(),
            [0x77, 0x55, 0, 0, 0, 5, 1, 2, 2, 0xa0, 8]
        );
        // read holdings
        holding_set(2, 9977).unwrap();
        holding_set(4, 9543).unwrap();
        holding_set(7, 9522).unwrap();
        let request = [1, 3, 0, 0, 0, 0xb];
        let frame = gen_tcp_frame(&request);
        let response = [
            0x77, 0x55, 0, 0, 0, 0x19, 1, 3, 0x16, 0, 0, 0, 0, 0x26, 0xf9, 0, 0, 0x25, 0x47, 0, 0,
            0, 0, 0x25, 0x32, 0, 0, 0, 0, 0, 0,
        ];
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // read inputs
        input_set(280, 99).unwrap();
        input_set(281, 15923).unwrap();
        input_set(284, 54321).unwrap();
        let frame = gen_tcp_frame(&[1, 4, 1, 0x18, 0, 6]);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(
            result.as_slice(),
            [
                0x77, 0x55, 0, 0, 0, 0xf, 1, 4, 0xc, 0, 0x63, 0x3e, 0x33, 0, 0, 0, 0, 0xd4, 0x31,
                0, 0
            ]
        );
    }

    #[test]
    fn test_std_frame_fc05_fc06() {
        clear_all();
        let mut result = Vec::new();
        // write coil
        let request = [1, 5, 0, 0xb, 0xff, 0];
        let response = [0x77, 0x55, 0, 0, 0, 6, 1, 5, 0, 0xb, 0xff, 0];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        assert_eq!(coil_get(11).unwrap(), true);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // write coil broadcast tcp
        let request = [0, 5, 0, 0x5, 0xff, 0];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.len(), 0);
        assert_eq!(coil_get(5).unwrap(), true);
        let request = [0, 5, 0, 0x7, 0xff, 0];
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        assert_eq!(result.len(), 0);
        assert_eq!(coil_get(7).unwrap(), true);
        // write coil invalid data
        let request = [1, 5, 0, 0xb, 0xff, 1];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x85, 3];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        // write coil context oob
        let request = [1, 5, 0x99, 0x99, 0xff, 0];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x85, 2];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // write holding
        let request = [1, 6, 0, 0xc, 0x33, 0x55];
        let response = [0x77, 0x55, 0, 0, 0, 6, 1, 6, 0, 0xc, 0x33, 0x55];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        assert_eq!(holding_get(12).unwrap(), 0x3355);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // write holding context oob
        let request = [1, 6, 0xff, 0xc, 0x33, 0x55];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x86, 2];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
    }

    #[test]
    fn test_std_frame_fc15() {
        clear_all();
        let mut result = Vec::new();
        // write multiple coils
        let request = [1, 0xf, 1, 0x31, 0, 5, 1, 0x25]; // 6 bits in data but 5 coils
        let response = [0x77, 0x55, 0, 0, 0, 6, 1, 0xf, 01, 0x31, 0, 5];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        assert_eq!(coil_get(305).unwrap(), true);
        assert_eq!(coil_get(306).unwrap(), false);
        assert_eq!(coil_get(307).unwrap(), true);
        assert_eq!(coil_get(308).unwrap(), false);
        assert_eq!(coil_get(309).unwrap(), false);
        assert_eq!(coil_get(310).unwrap(), false);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // write coils context oob
        let request = [1, 0xf, 0x99, 0xe8, 0, 5, 1, 0x25]; // 6 bits in data but 5 coils
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x8f, 2];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
    }

    #[test]
    fn test_std_frame_fc16() {
        clear_all();
        let mut result = Vec::new();
        // write multiple holdings
        let request = [
            1, 0x10, 1, 0x2c, 0, 4, 0x0a, 0x11, 0x22, 0x11, 0x33, 0x11, 0x55, 0x11, 0x99,
        ];
        let response = [0x77, 0x55, 0, 0, 0, 6, 1, 0x10, 1, 0x2c, 0, 4];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        assert_eq!(holding_get(300).unwrap(), 0x1122);
        assert_eq!(holding_get(301).unwrap(), 0x1133);
        assert_eq!(holding_get(302).unwrap(), 0x1155);
        assert_eq!(holding_get(303).unwrap(), 0x1199);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
        // write holdings context oob
        let request = [
            1, 0x10, 0x99, 0xe8, 0, 4, 0x0a, 0x11, 0x22, 0x11, 0x33, 0x11, 0x55, 0x11, 0x99,
        ];
        let response = [0x77, 0x55, 0, 0, 0, 3, 1, 0x90, 2];
        let frame = gen_tcp_frame(&request);
        process_frame(1, &frame, ModbusProto::TcpUdp, &mut result).unwrap();
        assert_eq!(result.as_slice(), response);
        let frame = gen_rtu_frame(&request);
        process_frame(1, &frame, ModbusProto::Rtu, &mut result).unwrap();
        check_rtu_response(&result, &response);
    }
}
