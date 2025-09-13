//! MODBUS Constants

/// MODBUS Functions
///
/// Some useful terminology:
///
/// - `Coil`: read/write, 1 bit
/// - `Discrete Input`: read-only, 1 bit
/// - `Input Register`: read-only, 16 bits (1 word)
/// - `Holding Register`: read/write, 16 bits (1 word)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ModbusFunction {
    /// Get Multiple Coils (Code = `0x01`)
    GetCoils = 0x01,
    /// Get Discrete inputs (Code = `0x02`)
    GetDiscretes = 0x02,
    /// Get Holdings (Code = `0x03`)
    GetHoldings = 0x03,
    /// Get Multiple Inputs registers (Code = `0x04`)
    GetInputs = 0x04,
    /// Set Single Coil (Code = `0x05`)
    SetCoil = 0x05,
    /// Set Single Holding Register (Code = `0x06`)
    SetHolding = 0x06,
    /// Set Coils Bulk (Code = `0x0F`)
    SetCoilsBulk = 0x0F,
    /// Set Holdings Bulk (Code = `0x10`)
    SetHoldingsBulk = 0x10,
}

impl TryFrom<u8> for ModbusFunction {
    type Error = crate::ErrorKind;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(ModbusFunction::GetCoils),
            0x02 => Ok(ModbusFunction::GetDiscretes),
            0x03 => Ok(ModbusFunction::GetHoldings),
            0x04 => Ok(ModbusFunction::GetInputs),
            0x05 => Ok(ModbusFunction::SetCoil),
            0x06 => Ok(ModbusFunction::SetHolding),
            0x0F => Ok(ModbusFunction::SetCoilsBulk),
            0x10 => Ok(ModbusFunction::SetHoldingsBulk),
            _ => Err(crate::ErrorKind::IllegalFunction),
        }
    }
}

impl ModbusFunction {
    /// Returns the function code that corresponds to this function as a byte.
    pub fn byte(&self) -> u8 {
        *self as u8
    }

    /// Returns whether this function is a read (`GET`) operation
    pub fn is_read(&self) -> bool {
        matches!(
            self,
            ModbusFunction::GetCoils
                | ModbusFunction::GetDiscretes
                | ModbusFunction::GetHoldings
                | ModbusFunction::GetInputs
        )
    }

    /// Returns whether this function is a write (`SET`) operation
    pub fn is_write(&self) -> bool {
        matches!(
            self,
            ModbusFunction::SetCoil
                | ModbusFunction::SetHolding
                | ModbusFunction::SetCoilsBulk
                | ModbusFunction::SetHoldingsBulk
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ModbusErrorCode {
    NoError = 0x00,
    IllegalFunction = 0x01,
    IllegalDataAddress = 0x02,
    IllegalDataValue = 0x03,
    SlaveDeviceFailure = 0x04,
    Acknowledge = 0x05,
    SlaveDeviceBusy = 0x06,
    NegativeAcknowledge = 0x07,
    MemoryParityError = 0x08,
    GatewayPathUnavailable = 0x09,
    GatewayTargetFailed = 0x0A,
    InvalidCrc = 0x15,
}

impl TryFrom<u8> for ModbusErrorCode {
    type Error = crate::ErrorKind;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(ModbusErrorCode::NoError),
            0x01 => Ok(ModbusErrorCode::IllegalFunction),
            0x02 => Ok(ModbusErrorCode::IllegalDataAddress),
            0x03 => Ok(ModbusErrorCode::IllegalDataValue),
            0x04 => Ok(ModbusErrorCode::SlaveDeviceFailure),
            0x05 => Ok(ModbusErrorCode::Acknowledge),
            0x06 => Ok(ModbusErrorCode::SlaveDeviceBusy),
            0x07 => Ok(ModbusErrorCode::NegativeAcknowledge),
            0x08 => Ok(ModbusErrorCode::MemoryParityError),
            0x09 => Ok(ModbusErrorCode::GatewayPathUnavailable),
            0x0A => Ok(ModbusErrorCode::GatewayTargetFailed),
            _ => Err(crate::ErrorKind::UnknownError),
        }
    }
}

impl ModbusErrorCode {
    /// Returns the error code as a byte.
    pub fn byte(&self) -> u8 {
        *self as u8
    }
}

// pub const MODBUS_GET_COILS: u8 = 1;
// pub const MODBUS_GET_DISCRETES: u8 = 2;
// pub const MODBUS_GET_HOLDINGS: u8 = 3;
// pub const MODBUS_GET_INPUTS: u8 = 4;
// pub const MODBUS_SET_COIL: u8 = 5;
// pub const MODBUS_SET_HOLDING: u8 = 6;
// pub const MODBUS_SET_COILS_BULK: u8 = 15;
// pub const MODBUS_SET_HOLDINGS_BULK: u8 = 16;

// // MODBUS Errors
// pub const MODBUS_ERROR_ILLEGAL_FUNCTION: u8 = 1;
// pub const MODBUS_ERROR_ILLEGAL_DATA_ADDRESS: u8 = 2;
// pub const MODBUS_ERROR_ILLEGAL_DATA_VALUE: u8 = 3;
