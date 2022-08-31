//! MODBUS Constants

// MODBUS Functions
pub const MODBUS_GET_COILS: u8 = 1;
pub const MODBUS_GET_DISCRETES: u8 = 2;
pub const MODBUS_GET_HOLDINGS: u8 = 3;
pub const MODBUS_GET_INPUTS: u8 = 4;
pub const MODBUS_SET_COIL: u8 = 5;
pub const MODBUS_SET_HOLDING: u8 = 6;
pub const MODBUS_SET_COILS_BULK: u8 = 15;
pub const MODBUS_SET_HOLDINGS_BULK: u8 = 16;

// MODBUS Errors
pub const MODBUS_ERROR_ILLEGAL_FUNCTION: u8 = 1;
pub const MODBUS_ERROR_ILLEGAL_DATA_ADDRESS: u8 = 2;
pub const MODBUS_ERROR_ILLEGAL_DATA_VALUE: u8 = 3;
