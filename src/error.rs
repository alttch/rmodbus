use core::num::TryFromIntError;

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
    ReadCallOnWriteFrame,
    WriteCallOnReadFrame,
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

    pub fn is_modbus_error(&self) -> bool {
        #[allow(clippy::enum_glob_use)]
        use ErrorKind::*;

        matches!(
            self,
            IllegalFunction
                | IllegalDataAddress
                | IllegalDataValue
                | SlaveDeviceFailure
                | Acknowledge
                | SlaveDeviceBusy
                | NegativeAcknowledge
                | MemoryParityError
                | GatewayPathUnavailable
                | GatewayTargetFailed
        )
    }

    pub fn to_modbus_error(&self) -> Result<u8, ErrorKind> {
        #[allow(clippy::enum_glob_use)]
        use ErrorKind::*;

        match self {
            IllegalFunction => Ok(1),
            IllegalDataAddress => Ok(2),
            IllegalDataValue => Ok(3),
            SlaveDeviceFailure => Ok(4),
            Acknowledge => Ok(5),
            SlaveDeviceBusy => Ok(6),
            NegativeAcknowledge => Ok(7),
            MemoryParityError => Ok(8),
            GatewayPathUnavailable => Ok(9),
            GatewayTargetFailed => Ok(10),
            _ => Err(*self),
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
            ErrorKind::ReadCallOnWriteFrame => {
                "FRAME DESCRIBING WRITE HAD FUNCTION CALLED FOR FRAMES DESCRIBING READ"
            }
            ErrorKind::WriteCallOnReadFrame => {
                "FRAME DESCRIBING READ HAD FUNCTION CALLED FOR FRAMES DESCRIBING WRITE"
            }
        };
        write!(f, "{}", msg)
    }
}

impl From<TryFromIntError> for ErrorKind {
    fn from(_: TryFromIntError) -> Self {
        ErrorKind::OOB
    }
}

impl core::error::Error for ErrorKind {}
