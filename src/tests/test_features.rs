use crate::server::ModbusFrame;
use crate::ModbusProto;

#[cfg(all(feature = "tcpudp", feature = "std"))]
#[test]
fn test_tcpudp() {
    let request = [];
    let mut result = Vec::new();
    ModbusFrame::new(1, &request, ModbusProto::TcpUdp, &mut result);
}

#[cfg(all(feature = "rtu", feature = "std"))]
#[test]
fn test_rtu() {
    let request = [];
    let mut result = Vec::new();
    ModbusFrame::new(1, &request, ModbusProto::Rtu, &mut result);
}

#[cfg(all(feature = "ascii", feature = "std"))]
#[test]
fn test_ascii() {
    let request = [];
    let mut result = Vec::new();
    ModbusFrame::new(1, &request, ModbusProto::Ascii, &mut result);
}
