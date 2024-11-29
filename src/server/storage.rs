use super::{
    super::{ErrorKind, VectorTrait},
    context::ModbusContext,
};
#[cfg(feature = "with_bincode")]
use bincode::{Decode, Encode};
use ieee754::Ieee754;
#[cfg(feature = "with_serde")]
use serde::{Deserialize, Serialize};

pub const SMALL_STORAGE_SIZE: usize = 1_000;
pub const FULL_STORAGE_SIZE: usize = 10_000;

/// Small context (1000) registers per type
pub type ModbusStorageSmall =
    ModbusStorage<SMALL_STORAGE_SIZE, SMALL_STORAGE_SIZE, SMALL_STORAGE_SIZE, SMALL_STORAGE_SIZE>;
/// Full context (10000) registers per type
pub type ModbusStorageFull =
    ModbusStorage<FULL_STORAGE_SIZE, FULL_STORAGE_SIZE, FULL_STORAGE_SIZE, FULL_STORAGE_SIZE>;

/// Contains standard Modbus register contexts
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "with_serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "with_bincode", derive(Decode, Encode))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModbusStorage<const C: usize, const D: usize, const I: usize, const H: usize> {
    #[cfg_attr(feature = "with_serde", serde(with = "serde_arrays"))]
    pub coils: [bool; C],
    #[cfg_attr(feature = "with_serde", serde(with = "serde_arrays"))]
    pub discretes: [bool; D],
    #[cfg_attr(feature = "with_serde", serde(with = "serde_arrays"))]
    pub inputs: [u16; I],
    #[cfg_attr(feature = "with_serde", serde(with = "serde_arrays"))]
    pub holdings: [u16; H],
}

macro_rules! get_regs_as_u8 {
    ($reg_context:expr, $reg:expr, $count:expr, $result:expr, $ctx_size: expr) => {{
        let reg_to = $reg as usize + $count as usize;
        if reg_to > $ctx_size {
            Err(ErrorKind::OOBContext)
        } else {
            for c in $reg as usize..reg_to {
                $result.push(($reg_context[c] >> 8) as u8)?;
                $result.push($reg_context[c] as u8)?;
            }
            Ok(())
        }
    }};
}

macro_rules! get_bools_as_u8 {
    ($reg_context:expr, $reg:expr, $count:expr, $result:expr, $ctx_size: expr) => {{
        let reg_to = $reg as usize + $count as usize;
        if reg_to > $ctx_size {
            Err(ErrorKind::OOBContext)
        } else {
            let mut creg = $reg as usize;
            while creg < reg_to {
                let mut cbyte = 0;
                for i in 0..8 {
                    if $reg_context[creg] {
                        cbyte |= 1 << i
                    }
                    creg += 1;
                    if creg >= reg_to {
                        break;
                    }
                }
                $result.push(cbyte)?;
            }
            Ok(())
        }
    }};
}

macro_rules! get_bools_as_u8_bytes {
    ($reg_context:expr, $reg:expr, $count:expr, $result:expr, $ctx_size: expr) => {{
        let reg_to = $reg as usize + $count as usize;
        if reg_to > $ctx_size {
            Err(ErrorKind::OOBContext)
        } else {
            for creg in $reg as usize..reg_to {
                $result.push(u8::from($reg_context[creg]))?;
            }
            Ok(())
        }
    }};
}

macro_rules! set_regs_from_u8 {
    ($reg_context:expr, $reg:expr, $values:expr, $ctx_size: expr) => {{
        let len = $values.len();
        if $reg as usize + len / 2 > $ctx_size {
            Err(ErrorKind::OOBContext)
        } else {
            let mut i = 0;
            let mut creg = $reg as usize;
            while i < len {
                $reg_context[creg] = u16::from_be_bytes([
                    $values[i],
                    match i + 1 < len {
                        true => $values[i + 1],
                        false => return Err(ErrorKind::OOB),
                    },
                ]);
                i += 2;
                creg += 1;
            }
            Ok(())
        }
    }};
}

macro_rules! set_bools_from_u8 {
    ($reg_context:expr, $reg:expr, $count:expr, $values:expr, $ctx_size: expr) => {{
        let reg_to = $reg as usize + $count as usize;
        if reg_to > $ctx_size {
            Err(ErrorKind::OOBContext)
        } else {
            let mut creg = $reg as usize;
            let mut cbyte = 0;
            let mut cnt = 0;
            let len = $values.len();
            while creg < reg_to && cnt < $count {
                if cbyte >= len {
                    return Err(ErrorKind::OOB);
                }
                let mut b: u8 = $values[cbyte];
                for _ in 0..8 {
                    $reg_context[creg] = b & 1 == 1;
                    b >>= 1;
                    creg += 1;
                    cnt += 1;
                    if cnt == $count || creg == reg_to {
                        break;
                    }
                }
                cbyte += 1;
            }
            Ok(())
        }
    }};
}

macro_rules! get_bulk {
    ($reg_context:expr, $reg:expr, $count:expr, $result:expr, $ctx_size: expr) => {{
        let reg_to = $reg as usize + $count as usize;
        if reg_to > $ctx_size {
            Err(ErrorKind::OOBContext)
        } else {
            $result.extend(&$reg_context[$reg as usize..reg_to])?;
            Ok(())
        }
    }};
}

macro_rules! set_bulk {
    ($reg_context:expr, $reg:expr, $values:expr, $ctx_size: expr) => {
        if $reg as usize + $values.len() > $ctx_size {
            Err(ErrorKind::OOBContext)
        } else {
            for (i, value) in $values.iter().enumerate() {
                $reg_context[$reg as usize + i] = *value;
            }
            Ok(())
        }
    };
}

macro_rules! set_bulk_bools_from_u8 {
    ($reg_context:expr, $reg:expr, $values:expr, $ctx_size: expr) => {
        if $reg as usize + $values.len() > $ctx_size {
            Err(ErrorKind::OOBContext)
        } else {
            for (i, value) in $values.iter().enumerate() {
                $reg_context[$reg as usize + i] = *value > 0;
            }
            Ok(())
        }
    };
}
macro_rules! get {
    ($reg_context:expr, $reg:expr, $ctx_size: expr) => {
        if $reg as usize >= $ctx_size {
            Err(ErrorKind::OOBContext)
        } else {
            Ok($reg_context[$reg as usize])
        }
    };
}

macro_rules! set {
    ($reg_context:expr, $reg:expr, $value:expr, $ctx_size: expr) => {
        if $reg as usize >= $ctx_size {
            Err(ErrorKind::OOBContext)
        } else {
            $reg_context[$reg as usize] = $value;
            Ok(())
        }
    };
}

macro_rules! get_u32 {
    ($reg_context:expr, $reg:expr, $ctx_size: expr) => {
        if $reg as usize + 1 >= $ctx_size {
            Err(ErrorKind::OOBContext)
        } else {
            Ok((($reg_context[$reg as usize] as u32) << 16)
                + $reg_context[($reg as usize) + 1] as u32)
        }
    };
}

macro_rules! set_u32 {
    ($reg_context:expr, $reg:expr, $value:expr, $ctx_size: expr) => {
        if $reg as usize + 1 >= $ctx_size {
            Err(ErrorKind::OOBContext)
        } else {
            $reg_context[$reg as usize] = ($value >> 16) as u16;
            $reg_context[$reg as usize + 1] = $value as u16;
            Ok(())
        }
    };
}

macro_rules! get_u64 {
    ($reg_context:expr, $reg:expr, $ctx_size: expr) => {
        if $reg as usize + 3 >= $ctx_size {
            Err(ErrorKind::OOBContext)
        } else {
            Ok((($reg_context[$reg as usize] as u64) << 48)
                + (($reg_context[$reg as usize + 1] as u64) << 32)
                + (($reg_context[$reg as usize + 2] as u64) << 16)
                + $reg_context[($reg as usize) + 3] as u64)
        }
    };
}

macro_rules! set_u64 {
    ($reg_context:expr, $reg:expr, $value:expr, $ctx_size: expr) => {
        if $reg as usize + 3 >= $ctx_size {
            Err(ErrorKind::OOBContext)
        } else {
            $reg_context[$reg as usize] = ($value >> 48) as u16;
            $reg_context[$reg as usize + 1] = ($value >> 32) as u16;
            $reg_context[$reg as usize + 2] = ($value >> 16) as u16;
            $reg_context[$reg as usize + 3] = $value as u16;
            Ok(())
        }
    };
}

impl<const C: usize, const D: usize, const I: usize, const H: usize> Default
    for ModbusStorage<C, D, I, H>
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const C: usize, const D: usize, const I: usize, const H: usize> ModbusStorage<C, D, I, H> {
    /// Define a custom-sized context
    ///
    /// The generic constants order is: coils, discretes, inputs, holdings
    ///
    /// E.g. let us define a context for 128 coils, 16 discretes, 0 inputs and 100 holdings:
    ///
    /// ```
    /// use rmodbus::server::storage::ModbusStorage;
    ///
    /// let context = ModbusStorage::<128, 16, 0, 100>::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            coils: [false; C],
            discretes: [false; D],
            inputs: [0; I],
            holdings: [0; H],
        }
    }

    pub fn clear_all(&mut self) {
        self.clear_coils();
        self.clear_discretes();
        self.clear_inputs();
        self.clear_holdings();
    }

    pub fn clear_coils(&mut self) {
        for i in 0..C {
            self.coils[i] = false;
        }
    }

    pub fn clear_discretes(&mut self) {
        for i in 0..D {
            self.discretes[i] = false;
        }
    }

    pub fn clear_inputs(&mut self) {
        for i in 0..I {
            self.inputs[i] = 0;
        }
    }

    pub fn clear_holdings(&mut self) {
        for i in 0..H {
            self.holdings[i] = 0;
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
impl<const C: usize, const D: usize, const I: usize, const H: usize> ModbusContext
    for ModbusStorage<C, D, I, H>
{
    fn get_inputs_as_u8<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_regs_as_u8!(self.inputs, reg, count, result, I)
    }

    fn get_holdings_as_u8<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_regs_as_u8!(self.holdings, reg, count, result, H)
    }

    fn set_inputs_from_u8(&mut self, reg: u16, values: &[u8]) -> Result<(), ErrorKind> {
        set_regs_from_u8!(self.inputs, reg, values, I)
    }

    fn set_holdings_from_u8(&mut self, reg: u16, values: &[u8]) -> Result<(), ErrorKind> {
        set_regs_from_u8!(self.holdings, reg, values, H)
    }

    fn get_coils_as_u8<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_bools_as_u8!(self.coils, reg, count, result, C)
    }

    fn get_coils_as_u8_bytes<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_bools_as_u8_bytes!(self.coils, reg, count, result, C)
    }

    fn get_discretes_as_u8<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_bools_as_u8!(self.discretes, reg, count, result, D)
    }

    fn get_discretes_as_u8_bytes<V: VectorTrait<u8>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_bools_as_u8_bytes!(self.discretes, reg, count, result, C)
    }

    fn set_coils_from_u8(&mut self, reg: u16, count: u16, values: &[u8]) -> Result<(), ErrorKind> {
        set_bools_from_u8!(self.coils, reg, count, values, C)
    }

    fn set_coils_from_u8_bytes(&mut self, reg: u16, values: &[u8]) -> Result<(), ErrorKind> {
        set_bulk_bools_from_u8!(self.coils, reg, values, C)
    }

    fn set_discretes_from_u8_bytes(&mut self, reg: u16, values: &[u8]) -> Result<(), ErrorKind> {
        set_bulk_bools_from_u8!(self.discretes, reg, values, D)
    }

    fn set_discretes_from_u8(
        &mut self,
        reg: u16,
        count: u16,
        values: &[u8],
    ) -> Result<(), ErrorKind> {
        set_bools_from_u8!(self.discretes, reg, count, values, D)
    }

    fn get_coils_bulk<V: VectorTrait<bool>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_bulk!(self.coils, reg, count, result, C)
    }

    fn get_discretes_bulk<V: VectorTrait<bool>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_bulk!(self.discretes, reg, count, result, D)
    }

    fn get_inputs_bulk<V: VectorTrait<u16>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_bulk!(self.inputs, reg, count, result, I)
    }

    fn get_holdings_bulk<V: VectorTrait<u16>>(
        &self,
        reg: u16,
        count: u16,
        result: &mut V,
    ) -> Result<(), ErrorKind> {
        get_bulk!(self.holdings, reg, count, result, H)
    }

    fn set_coils_bulk(&mut self, reg: u16, values: &[bool]) -> Result<(), ErrorKind> {
        set_bulk!(self.coils, reg, values, C)
    }

    fn set_discretes_bulk(&mut self, reg: u16, values: &[bool]) -> Result<(), ErrorKind> {
        set_bulk!(self.discretes, reg, values, D)
    }

    fn set_inputs_bulk(&mut self, reg: u16, values: &[u16]) -> Result<(), ErrorKind> {
        set_bulk!(self.inputs, reg, values, I)
    }

    fn set_holdings_bulk(&mut self, reg: u16, values: &[u16]) -> Result<(), ErrorKind> {
        set_bulk!(self.holdings, reg, values, H)
    }

    fn get_coil(&self, reg: u16) -> Result<bool, ErrorKind> {
        get!(self.coils, reg, C)
    }

    fn get_discrete(&self, reg: u16) -> Result<bool, ErrorKind> {
        get!(self.discretes, reg, D)
    }

    fn get_input(&self, reg: u16) -> Result<u16, ErrorKind> {
        get!(self.inputs, reg, I)
    }

    fn get_holding(&self, reg: u16) -> Result<u16, ErrorKind> {
        get!(self.holdings, reg, H)
    }

    fn set_coil(&mut self, reg: u16, value: bool) -> Result<(), ErrorKind> {
        set!(self.coils, reg, value, C)
    }

    fn set_discrete(&mut self, reg: u16, value: bool) -> Result<(), ErrorKind> {
        set!(self.discretes, reg, value, D)
    }

    fn set_input(&mut self, reg: u16, value: u16) -> Result<(), ErrorKind> {
        set!(self.inputs, reg, value, I)
    }

    fn set_holding(&mut self, reg: u16, value: u16) -> Result<(), ErrorKind> {
        set!(self.holdings, reg, value, H)
    }

    fn get_inputs_as_u32(&self, reg: u16) -> Result<u32, ErrorKind> {
        get_u32!(self.inputs, reg, I)
    }

    fn get_holdings_as_u32(&self, reg: u16) -> Result<u32, ErrorKind> {
        get_u32!(self.holdings, reg, H)
    }

    fn set_inputs_from_u32(&mut self, reg: u16, value: u32) -> Result<(), ErrorKind> {
        set_u32!(self.inputs, reg, value, I)
    }

    fn set_holdings_from_u32(&mut self, reg: u16, value: u32) -> Result<(), ErrorKind> {
        set_u32!(self.holdings, reg, value, H)
    }

    fn get_inputs_as_u64(&self, reg: u16) -> Result<u64, ErrorKind> {
        get_u64!(self.inputs, reg, I)
    }

    fn get_holdings_as_u64(&self, reg: u16) -> Result<u64, ErrorKind> {
        get_u64!(self.holdings, reg, H)
    }

    fn set_inputs_from_u64(&mut self, reg: u16, value: u64) -> Result<(), ErrorKind> {
        set_u64!(self.inputs, reg, value, I)
    }

    fn set_holdings_from_u64(&mut self, reg: u16, value: u64) -> Result<(), ErrorKind> {
        set_u64!(self.holdings, reg, value, H)
    }

    fn get_inputs_as_f32(&self, reg: u16) -> Result<f32, ErrorKind> {
        Ok(Ieee754::from_bits(self.get_inputs_as_u32(reg)?))
    }

    fn get_holdings_as_f32(&self, reg: u16) -> Result<f32, ErrorKind> {
        Ok(Ieee754::from_bits(self.get_holdings_as_u32(reg)?))
    }

    #[inline]
    fn set_inputs_from_f32(&mut self, reg: u16, value: f32) -> Result<(), ErrorKind> {
        self.set_inputs_from_u32(reg, value.bits())
    }

    #[inline]
    fn set_holdings_from_f32(&mut self, reg: u16, value: f32) -> Result<(), ErrorKind> {
        self.set_holdings_from_u32(reg, value.bits())
    }
}
