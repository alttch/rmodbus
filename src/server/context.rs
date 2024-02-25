use crate::{ErrorKind, VectorTrait};

#[allow(clippy::module_name_repetitions)]
pub trait ModbusContext {
    /// Get inputs as Vec of u8
    ///
    /// Note: Vec is always appended
    fn get_inputs_as_u8<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind>;

    /// Get holdings as Vec of u8
    ///
    /// Note: Vec is always appended
    fn get_holdings_as_u8<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind>;

    /// Set inputs from Vec of u8
    fn set_inputs_from_u8(&mut self, reg: u16, values: &[u8]) -> Result<(), ErrorKind>;

    /// Set holdings from Vec of u8
    fn set_holdings_from_u8(&mut self, reg: u16, values: &[u8]) -> Result<(), ErrorKind>;

    /// Get coils as Vec of u8
    ///
    /// Note: Vec is always appended
    fn get_coils_as_u8<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind>;

    /// Get discretes as Vec of u8
    ///
    /// Note: Vec is always appended
    fn get_discretes_as_u8<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind>;

    /// Set coils from Vec of u8
    ///
    /// As coils are packed in u8, parameter *count* specifies how many coils are actually needed
    /// to set, extra bits are ignored
    fn set_coils_from_u8(&mut self, reg: u16, count: u16, values: &[u8]) -> Result<(), ErrorKind>;

    /// Set discretes from Vec of u8
    ///
    /// As discretes are packed in u8, parameter *count* specifies how many coils are actually
    /// needed to set, extra bits are ignored
    fn set_discretes_from_u8(
        &mut self,
        reg: u16,
        count: u16,
        values: &[u8],
    ) -> Result<(), ErrorKind>;

    /// Bulk get coils
    ///
    /// Note: Vec is always appended
    fn get_coils_bulk<V: VectorTrait<bool>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind>;

    /// Bulk get discretes
    ///
    /// Note: Vec is always appended
    fn get_discretes_bulk<V: VectorTrait<bool>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind>;

    /// Bulk get inputs
    ///
    /// Note: Vec is always appended
    fn get_inputs_bulk<V: VectorTrait<u16>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind>;

    /// Bulk get holdings
    ///
    /// Note: Vec is always appended
    fn get_holdings_bulk<V: VectorTrait<u16>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind>;

    /// Bulk set coils
    fn set_coils_bulk(&mut self, reg: u16, values: &[bool]) -> Result<(), ErrorKind>;

    /// Bulk set discretes
    fn set_discretes_bulk(&mut self, reg: u16, values: &[bool]) -> Result<(), ErrorKind>;

    /// Bulk set inputs
    fn set_inputs_bulk(&mut self, reg: u16, values: &[u16]) -> Result<(), ErrorKind>;

    /// Bulk set holdings
    fn set_holdings_bulk(&mut self, reg: u16, values: &[u16]) -> Result<(), ErrorKind>;

    /// Get a single coil
    fn get_coil(&self, reg: u16) -> Result<bool, ErrorKind>;

    /// Get a single discrete
    fn get_discrete(&self, reg: u16) -> Result<bool, ErrorKind>;

    /// Get a single input
    fn get_input(&self, reg: u16) -> Result<u16, ErrorKind>;

    /// Get a single holding
    fn get_holding(&self, reg: u16) -> Result<u16, ErrorKind>;

    /// Set a single coil
    fn set_coil(&mut self, reg: u16, value: bool) -> Result<(), ErrorKind>;

    /// Set a single discrete
    fn set_discrete(&mut self, reg: u16, value: bool) -> Result<(), ErrorKind>;

    /// Set a single input
    fn set_input(&mut self, reg: u16, value: u16) -> Result<(), ErrorKind>;

    /// Set a single holding
    fn set_holding(&mut self, reg: u16, value: u16) -> Result<(), ErrorKind>;

    /// Get two inputs as u32
    ///
    /// Returns 32-bit value (big-endian)
    fn get_inputs_as_u32(&self, reg: u16) -> Result<u32, ErrorKind>;

    /// Get two holdings as u32
    ///
    /// Returns 32-bit value (big-endian)
    fn get_holdings_as_u32(&self, reg: u16) -> Result<u32, ErrorKind>;

    /// Set two inputs from u32
    ///
    /// Uses 32-bit value to set two registers (big-endian)
    fn set_inputs_from_u32(&mut self, reg: u16, value: u32) -> Result<(), ErrorKind>;

    /// Set two holdings from u32
    ///
    /// Uses 32-bit value to set two registers (big-endian)
    fn set_holdings_from_u32(&mut self, reg: u16, value: u32) -> Result<(), ErrorKind>;

    /// Get four inputs as u64
    ///
    /// Returns 64-bit value (big-endian)
    fn get_inputs_as_u64(&self, reg: u16) -> Result<u64, ErrorKind>;

    /// Get four holdings as u64
    ///
    /// Returns 64-bit value (big-endian)
    fn get_holdings_as_u64(&self, reg: u16) -> Result<u64, ErrorKind>;

    /// Set four inputs from u64
    ///
    /// Uses 64-bit value to set four registers (big-endian)
    fn set_inputs_from_u64(&mut self, reg: u16, value: u64) -> Result<(), ErrorKind>;

    /// Set four holdings from u64
    ///
    /// Uses 64-bit value to set four registers (big-endian)
    fn set_holdings_from_u64(&mut self, reg: u16, value: u64) -> Result<(), ErrorKind>;

    /// Get two input registers as IEEE754 32-bit float
    fn get_inputs_as_f32(&self, reg: u16) -> Result<f32, ErrorKind>;

    /// Get two holding registers as IEEE754 32-bit float
    fn get_holdings_as_f32(&self, reg: u16) -> Result<f32, ErrorKind>;

    /// Set IEEE 754 f32 to two input registers
    fn set_inputs_from_f32(&mut self, reg: u16, value: f32) -> Result<(), ErrorKind>;

    /// Set IEEE 754 f32 to two holding registers
    fn set_holdings_from_f32(&mut self, reg: u16, value: f32) -> Result<(), ErrorKind>;
}
