#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use rmodbus::{
    server::{storage::ModbusStorageFull, ModbusFrame},
    ModbusProto,
};
use std::vec::Vec;

#[derive(Debug, Arbitrary)]
enum FuzzProto {
    Rtu,
    Ascii,
    TcpUdp,
}

impl From<FuzzProto> for ModbusProto {
    fn from(p: FuzzProto) -> Self {
        match p {
            FuzzProto::Rtu => ModbusProto::Rtu,
            FuzzProto::Ascii => ModbusProto::Ascii,
            FuzzProto::TcpUdp => ModbusProto::TcpUdp,
        }
    }
}

#[derive(Debug, Arbitrary)]
struct FuzzInput<'a> {
    unit_id: u8,
    proto: FuzzProto,
    request_buf: &'a [u8],
}

fuzz_target!(|data: FuzzInput| {
    // we only care about panics so we can ignore results
    let _ = fuzz_server(data);
});

fn fuzz_server(input: FuzzInput) -> Result<(), rmodbus::ErrorKind> {
    let mut response_buf = Vec::new();
    let mut frame = ModbusFrame::new(
        input.unit_id,
        input.request_buf,
        input.proto.into(),
        &mut response_buf,
    );

    if frame.parse().is_ok() {
        if frame.processing_required {
            let mut context = ModbusStorageFull::new();
            match frame.readonly {
                true => {
                    let _ = frame.process_read(&context);
                }
                false => {
                    let _ = frame.process_write(&mut context);
                }
            }
        }

        if frame.response_required {
            let _ = frame.finalize_response();
        }
    }

    Ok(())
}
