#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ErrorKind {
    OOB,
    OOBContext,
    FrameBroken,
    FrameCRCError,
    IllegalFunction,
    IllegalDataAddress,
    IllegalDataValue,
    SlaveDeviceFailure,
    Acknowledge,
    SlaveDeviceBusy,
    NegativeAcknowledge,
    MemoryParityError,
    GatewayPathUnavailable,
    GatewayTargetFailed,
    CommunicationError,
    UnknownError,
    Utf8Error,
}

impl ErrorKind {
    pub fn from_modbus_error(code: u8) -> Self {
        match code {
            0x01 => ErrorKind::IllegalFunction,
            0x02 => ErrorKind::IllegalDataAddress,
            0x03 => ErrorKind::IllegalDataValue,
            0x04 => ErrorKind::SlaveDeviceFailure,
            0x05 => ErrorKind::Acknowledge,
            0x06 => ErrorKind::SlaveDeviceBusy,
            0x07 => ErrorKind::NegativeAcknowledge,
            0x08 => ErrorKind::MemoryParityError,
            0x09 => ErrorKind::GatewayPathUnavailable,
            0x10 => ErrorKind::GatewayTargetFailed,
            _ => ErrorKind::UnknownError,
        }
    }
}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg: &str = match self {
            ErrorKind::OOB => "OUT OF BUFFER",
            ErrorKind::OOBContext => "OUT OF BUFFER IN CONTEXT",
            ErrorKind::FrameBroken => "FRAME BROKEN",
            ErrorKind::FrameCRCError => "FRAME CRC ERROR",
            ErrorKind::IllegalFunction => "MODBUS ERROR CODE 01 - ILLEGAL FUNCTION",
            ErrorKind::IllegalDataAddress => "MODBUS ERROR CODE 02 - ILLEGAL DATA ADDRESS",
            ErrorKind::IllegalDataValue => "MODBUS ERROR CODE 03 - ILLEGAL DATA VALUE",
            ErrorKind::SlaveDeviceFailure => "MODBUS ERROR CODE 04 - SLAVE DEVICE FAILURE",
            ErrorKind::Acknowledge => "MODBUS ERROR CODE 05 - ACKNOWLEDGE",
            ErrorKind::SlaveDeviceBusy => "MODBUS ERROR CODE 06 - SLAVE DEVICE BUSY",
            ErrorKind::NegativeAcknowledge => "MODBUS ERROR CODE 07 - NEGATIVE ACKNOWLEDGE",
            ErrorKind::MemoryParityError => "MODBUS ERROR CODE 08 - MEMORY PARITY ERROR",
            ErrorKind::GatewayPathUnavailable => "MODBUS ERROR CODE 10 - GATEWAY PATH UNAVAILABLE",
            ErrorKind::GatewayTargetFailed => {
                "MODBUS ERROR CODE 11 - GATEWAY TARGET DEVICE FAILED TO RESPOND"
            }
            ErrorKind::CommunicationError => {
                "MODBUS ERROR CODE 21 - Response CRC did not match calculated CRC"
            }
            ErrorKind::UnknownError => "UNKNOWN MODBUS ERROR",
            ErrorKind::Utf8Error => "UTF8 CONVERTION ERROR",
        };
        write!(f, "{}", msg)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ErrorKind {}
