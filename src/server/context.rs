use super::representable::RegisterRepresentable;
use crate::{ErrorKind, VectorTrait};
use ieee754::Ieee754;

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
    ) -> Result<(), ErrorKind> {
        for i in reg..(reg + count) {
            let input = self.get_input(i)?;
            result.push((input >> 8) as u8)?;
            result.push(input as u8)?;
        }
        Ok(())
    }

    /// Get holdings as Vec of u8
    ///
    /// Note: Vec is always appended
    fn get_holdings_as_u8<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        for i in reg..(reg + count) {
            let holding = self.get_holding(i)?;
            result.push((holding >> 8) as u8)?;
            result.push(holding as u8)?;
        }
        Ok(())
    }

    /// Set inputs from Vec of u8
    fn set_inputs_from_u8(&mut self, reg: u16, values: &[u8]) -> Result<(), ErrorKind> {
        let mut current_reg = reg;
        let mut chunk_iter = values.chunks_exact(2);
        for pair in chunk_iter.by_ref() {
            let reg_value = ((pair[0] as u16) << 8) + pair[1] as u16;
            self.set_input(current_reg, reg_value)?;
            current_reg += 1;
        }
        let remaining = chunk_iter.remainder();
        if !remaining.is_empty() {
            // 1 u8 left
            let reg_value = (remaining[0] as u16) << 8;
            self.set_input(current_reg, reg_value)?;
        }
        Ok(())
    }

    /// Set holdings from Vec of u8
    fn set_holdings_from_u8(&mut self, reg: u16, values: &[u8]) -> Result<(), ErrorKind> {
        let mut current_reg = reg;
        let mut chunk_iter = values.chunks_exact(2);
        for pair in chunk_iter.by_ref() {
            let reg_value = ((pair[0] as u16) << 8) + pair[1] as u16;
            self.set_holding(current_reg, reg_value)?;
            current_reg += 1;
        }
        let remaining = chunk_iter.remainder();
        if !remaining.is_empty() {
            // 1 u8 left
            let reg_value = (remaining[0] as u16) << 8;
            self.set_holding(current_reg, reg_value)?;
        }
        Ok(())
    }

    /// Get coils as Vec of u8 (packed as BITS, 1 byte = 8 coils)
    ///
    /// Note: Vec is always appended
    fn get_coils_as_u8<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        let mut creg = reg;
        while creg < (reg + count) {
            let mut cbyte = 0;
            for i in 0..8 {
                if self.get_coil(creg)? {
                    cbyte |= 1 << i
                }
                creg += 1;
                if creg >= (reg + count) {
                    break;
                }
            }
            result.push(cbyte)?;
        }
        Ok(())
    }

    /// Get coils as Vec of u8 (packed as BYTES)
    ///
    /// Note: Vec is always appended
    fn get_coils_as_u8_bytes<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        for i in reg..(reg + count) {
            result.push(u8::from(self.get_coil(i)?))?;
        }
        Ok(())
    }

    /// Get discretes as Vec of u8 (packed as BITS, 1 byte = 8 discretes)
    ///
    /// Note: Vec is always appended
    fn get_discretes_as_u8<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        let mut creg = reg;
        while creg < (reg + count) {
            let mut cbyte = 0;
            for i in 0..8 {
                if self.get_discrete(creg)? {
                    cbyte |= 1 << i
                }
                creg += 1;
                if creg >= (reg + count) {
                    break;
                }
            }
            result.push(cbyte)?;
        }
        Ok(())
    }

    /// Get discretes as Vec of u8 (packed as BYTES)
    ///
    /// Note: Vec is always appended
    fn get_discretes_as_u8_bytes<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        for i in reg..(reg + count) {
            result.push(u8::from(self.get_discrete(i)?))?;
        }
        Ok(())
    }

    /// Set coils from Vec of u8 (packed as BITS, 1 byte = 8 coils)
    ///
    /// As coils are packed in u8, parameter *count* specifies how many coils are actually needed
    /// to set, extra bits are ignored
    fn set_coils_from_u8(&mut self, reg: u16, count: u16, values: &[u8]) -> Result<(), ErrorKind> {
        let bit_iter = values
            .iter()
            .flat_map(|v| (0..8).map(move |i| (v >> i) & 1 == 1));
        for (reg_index, bit) in bit_iter
            .take(count as usize)
            .enumerate()
            .map(|(i, b)| (i as u16 + reg, b))
        {
            self.set_coil(reg_index, bit)?;
        }
        if count >= (values.len() as u16) * 8 {
            Err(ErrorKind::OOB)
        } else {
            Ok(())
        }
    }

    /// Set discretes from Vec of u8 (packed as BITS, 1 byte = 8 discretes)
    ///
    /// As discretes are packed in u8, parameter *count* specifies how many coils are actually
    /// needed to set, extra bits are ignored
    fn set_discretes_from_u8(
        &mut self,
        reg: u16,
        count: u16,
        values: &[u8],
    ) -> Result<(), ErrorKind> {
        let bit_iter = values
            .iter()
            .flat_map(|v| (0..8).map(move |i| (v >> i) & 1 == 1));
        for (reg_index, bit) in bit_iter
            .take(count as usize)
            .enumerate()
            .map(|(i, b)| (i as u16 + reg, b))
        {
            self.set_discrete(reg_index, bit)?;
        }
        if count >= (values.len() as u16) * 8 {
            Err(ErrorKind::OOB)
        } else {
            Ok(())
        }
    }

    /// Set coils from Vec of u8 (packed as BYTES)
    fn set_coils_from_u8_bytes(&mut self, reg: u16, values: &[u8]) -> Result<(), ErrorKind> {
        for (i, value) in values.iter().enumerate().map(|(i, v)| (i as u16 + reg, v)) {
            self.set_coil(i, *value > 0)?;
        }
        Ok(())
    }

    /// Set discretes from Vec of u8 (packed as BYTES)
    fn set_discretes_from_u8_bytes(&mut self, reg: u16, values: &[u8]) -> Result<(), ErrorKind> {
        for (i, value) in values.iter().enumerate().map(|(i, v)| (i as u16 + reg, v)) {
            self.set_discrete(i, *value > 0)?;
        }
        Ok(())
    }

    /// Bulk get coils
    ///
    /// Note: Vec is always appended
    fn get_coils_bulk<V: VectorTrait<bool>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        for i in reg..(reg + count) {
            result.push(self.get_coil(i)?)?;
        }
        Ok(())
    }

    /// Bulk get discretes
    ///
    /// Note: Vec is always appended
    fn get_discretes_bulk<V: VectorTrait<bool>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        for i in reg..(reg + count) {
            result.push(self.get_discrete(i)?)?;
        }
        Ok(())
    }

    /// Bulk get inputs
    ///
    /// Note: Vec is always appended
    fn get_inputs_bulk<V: VectorTrait<u16>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        for i in reg..(reg + count) {
            result.push(self.get_input(i)?)?;
        }
        Ok(())
    }

    /// Bulk get holdings
    ///
    /// Note: Vec is always appended
    fn get_holdings_bulk<V: VectorTrait<u16>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        for i in reg..(reg + count) {
            result.push(self.get_holding(i)?)?;
        }
        Ok(())
    }

    /// Bulk set coils
    fn set_coils_bulk(&mut self, reg: u16, values: &[bool]) -> Result<(), ErrorKind> {
        for (i, value) in values.iter().enumerate().map(|(i, v)| (i as u16 + reg, v)) {
            self.set_coil(i, *value)?;
        }
        Ok(())
    }

    /// Bulk set discretes
    fn set_discretes_bulk(&mut self, reg: u16, values: &[bool]) -> Result<(), ErrorKind> {
        for (i, value) in values.iter().enumerate().map(|(i, v)| (i as u16 + reg, v)) {
            self.set_discrete(i, *value)?;
        }
        Ok(())
    }

    /// Bulk set inputs
    fn set_inputs_bulk(&mut self, reg: u16, values: &[u16]) -> Result<(), ErrorKind> {
        for (i, value) in values.iter().enumerate().map(|(i, v)| (i as u16 + reg, v)) {
            self.set_input(i, *value)?;
        }
        Ok(())
    }

    /// Bulk set holdings
    fn set_holdings_bulk(&mut self, reg: u16, values: &[u16]) -> Result<(), ErrorKind> {
        for (i, value) in values.iter().enumerate().map(|(i, v)| (i as u16 + reg, v)) {
            self.set_holding(i, *value)?;
        }
        Ok(())
    }

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
    fn get_inputs_as_u32(&self, reg: u16) -> Result<u32, ErrorKind> {
        Ok(((self.get_input(reg)? as u32) << 16) + (self.get_input(reg + 1)? as u32))
    }

    /// Get two holdings as u32
    ///
    /// Returns 32-bit value (big-endian)
    fn get_holdings_as_u32(&self, reg: u16) -> Result<u32, ErrorKind> {
        Ok(((self.get_holding(reg)? as u32) << 16) + (self.get_holding(reg + 1)? as u32))
    }

    /// Set two inputs from u32
    ///
    /// Uses 32-bit value to set two registers (big-endian)
    fn set_inputs_from_u32(&mut self, reg: u16, value: u32) -> Result<(), ErrorKind> {
        self.set_input(reg, (value >> 16) as u16)?;
        self.set_input(reg + 1, value as u16)?;
        Ok(())
    }

    /// Set two holdings from u32
    ///
    /// Uses 32-bit value to set two registers (big-endian)
    fn set_holdings_from_u32(&mut self, reg: u16, value: u32) -> Result<(), ErrorKind> {
        self.set_holding(reg, (value >> 16) as u16)?;
        self.set_holding(reg + 1, value as u16)?;
        Ok(())
    }

    /// Get four inputs as u64
    ///
    /// Returns 64-bit value (big-endian)
    fn get_inputs_as_u64(&self, reg: u16) -> Result<u64, ErrorKind> {
        Ok(((self.get_input(reg)? as u64) << 48)
            + ((self.get_input(reg)? as u64) << 32)
            + ((self.get_input(reg)? as u64) << 16)
            + (self.get_input(reg)? as u64))
    }

    /// Get four holdings as u64
    ///
    /// Returns 64-bit value (big-endian)
    fn get_holdings_as_u64(&self, reg: u16) -> Result<u64, ErrorKind> {
        Ok(((self.get_holding(reg)? as u64) << 48)
            + ((self.get_holding(reg)? as u64) << 32)
            + ((self.get_holding(reg)? as u64) << 16)
            + (self.get_holding(reg)? as u64))
    }

    /// Set four inputs from u64
    ///
    /// Uses 64-bit value to set four registers (big-endian)
    fn set_inputs_from_u64(&mut self, reg: u16, value: u64) -> Result<(), ErrorKind> {
        self.set_input(reg, (value >> 48) as u16)?;
        self.set_input(reg + 1, (value >> 32) as u16)?;
        self.set_input(reg + 2, (value >> 16) as u16)?;
        self.set_input(reg + 3, value as u16)?;
        Ok(())
    }

    /// Set four holdings from u64
    ///
    /// Uses 64-bit value to set four registers (big-endian)
    fn set_holdings_from_u64(&mut self, reg: u16, value: u64) -> Result<(), ErrorKind> {
        self.set_holding(reg, (value >> 48) as u16)?;
        self.set_holding(reg + 1, (value >> 32) as u16)?;
        self.set_holding(reg + 2, (value >> 16) as u16)?;
        self.set_holding(reg + 3, value as u16)?;
        Ok(())
    }

    /// Get two input registers as IEEE754 32-bit float
    fn get_inputs_as_f32(&self, reg: u16) -> Result<f32, ErrorKind> {
        Ok(Ieee754::from_bits(self.get_inputs_as_u32(reg)?))
    }

    /// Get two holding registers as IEEE754 32-bit float
    fn get_holdings_as_f32(&self, reg: u16) -> Result<f32, ErrorKind> {
        Ok(Ieee754::from_bits(self.get_holdings_as_u32(reg)?))
    }

    /// Set IEEE 754 f32 to two input registers
    fn set_inputs_from_f32(&mut self, reg: u16, value: f32) -> Result<(), ErrorKind> {
        self.set_inputs_from_u32(reg, value.bits())
    }

    /// Set IEEE 754 f32 to two holding registers
    fn set_holdings_from_f32(&mut self, reg: u16, value: f32) -> Result<(), ErrorKind> {
        self.set_holdings_from_u32(reg, value.bits())
    }

    /// Get N inputs represented as some [`RegisterRepresentable`] type T
    ///
    /// Returns the [`RegisterRepresentable`] once converted using
    /// [`RegisterRepresentable::from_registers_sequential`]
    #[allow(clippy::cast_possible_truncation)]
    fn get_inputs_as_representable<const N: usize, T: RegisterRepresentable<N>>(
        &self,
        reg: u16,
    ) -> Result<T, ErrorKind> {
        let mut regs: [u16; N] = [0u16; N];
        for (i, r) in regs.iter_mut().enumerate().take(N) {
            *r = self.get_input(reg + i as u16)?;
        }
        Ok(T::from_registers_sequential(&regs))
    }

    /// Get N holdings represented as some [`RegisterRepresentable`] type T.
    ///
    /// Returns the [`RegisterRepresentable`] once converted using
    /// [`RegisterRepresentable::from_registers_sequential`]
    #[allow(clippy::cast_possible_truncation)]
    fn get_holdings_as_representable<const N: usize, T: RegisterRepresentable<N>>(
        &self,
        reg: u16,
    ) -> Result<T, ErrorKind> {
        let mut regs: [u16; N] = [0u16; N];
        for (i, r) in regs.iter_mut().enumerate().take(N) {
            *r = self.get_holding(reg + i as u16)?;
        }
        Ok(T::from_registers_sequential(&regs))
    }

    /// Set N inputs using a [`RegisterRepresentable`].
    ///
    /// Uses [`RegisterRepresentable::to_registers_sequential`] to convert
    /// type T into a sequence of [`u16`] registers.
    fn set_inputs_from_representable<const N: usize, T: RegisterRepresentable<N>>(
        &mut self,
        reg: u16,
        value: &T,
    ) -> Result<(), ErrorKind> {
        let regs = value.to_registers_sequential();
        self.set_inputs_bulk(reg, &regs)
    }

    /// Set N holdings using a [`RegisterRepresentable`].
    ///
    /// Uses [`RegisterRepresentable::to_registers_sequential`] to convert
    /// type T into a sequence of [`u16`] registers.
    fn set_holdings_from_representable<const N: usize, T: RegisterRepresentable<N>>(
        &mut self,
        reg: u16,
        value: &T,
    ) -> Result<(), ErrorKind> {
        let regs = value.to_registers_sequential();
        self.set_holdings_bulk(reg, &regs)
    }
}
