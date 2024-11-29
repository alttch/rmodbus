/// Implemented for structs that can be represented using N u16 registers.
/// It is highly recommended that implementors of this type ensure that
/// [`RegisterRepresentable::to_registers_sequential`] and
/// [`RegisterRepresentable::from_registers_sequential`] are exact
/// inverses of each other.
pub trait RegisterRepresentable<const N: usize> {
    /// Convert this type into a sequence of `u16`s which can be loaded
    /// into modbus registers.
    fn to_registers_sequential(&self) -> [u16; N];
    /// Extract this type from a sequence of `u16`s taken from sequential
    /// modbus registers.
    fn from_registers_sequential(value: &[u16; N]) -> Self;
}

/// Some implementations of [`RegisterRepresentable`] for convenience.
/// You can implement [`RegisterRepresentable`] for your custom types
/// if they aren't implemented here.
pub mod representations {
    use super::RegisterRepresentable;
    #[cfg(feature = "with_bincode")]
    use bincode::{Decode, Encode};
    #[cfg(feature = "with_serde")]
    use serde::{Deserialize, Serialize};

    /// A [`u32`] represented in 2 [`u16`] registers with big endian.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[cfg_attr(feature = "with_serde", derive(Deserialize, Serialize))]
    #[cfg_attr(feature = "with_bincode", derive(Decode, Encode))]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct U32BigEndian(pub u32);
    impl RegisterRepresentable<2> for U32BigEndian {
        fn to_registers_sequential(&self) -> [u16; 2] {
            [(self.0 >> 16) as u16, self.0 as u16]
        }
        fn from_registers_sequential(value: &[u16; 2]) -> Self {
            Self(((value[0] as u32) << 16) | (value[1] as u32))
        }
    }

    /// A [`u32`] represented in 2 [`u16`] registers with little endian.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[cfg_attr(feature = "with_serde", derive(Deserialize, Serialize))]
    #[cfg_attr(feature = "with_bincode", derive(Decode, Encode))]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct U32LittleEndian(pub u32);
    impl RegisterRepresentable<2> for U32LittleEndian {
        fn to_registers_sequential(&self) -> [u16; 2] {
            [self.0 as u16, (self.0 >> 16) as u16]
        }
        fn from_registers_sequential(value: &[u16; 2]) -> Self {
            Self((value[0] as u32) | ((value[1] as u32) << 16))
        }
    }

    /// A [`u64`] represented in 2 [`u16`] registers with big endian.
    pub struct U64BigEndian(pub u64);
    impl RegisterRepresentable<4> for U64BigEndian {
        fn to_registers_sequential(&self) -> [u16; 4] {
            [
                ((self.0 & 0xFFFF_0000_0000_0000) >> 48) as u16,
                ((self.0 & 0x0000_FFFF_0000_0000) >> 32) as u16,
                ((self.0 & 0x0000_0000_FFFF_0000) >> 16) as u16,
                self.0 as u16,
            ]
        }
        fn from_registers_sequential(value: &[u16; 4]) -> Self {
            Self(
                (value[0] as u64) << 48
                    | (value[1] as u64) << 32
                    | (value[2] as u64) << 16
                    | (value[3] as u64),
            )
        }
    }
    /// A [`u64`] represented in 2 [`u16`] registers with little endian.
    pub struct U64LittleEndian(pub u64);
    impl RegisterRepresentable<4> for U64LittleEndian {
        fn to_registers_sequential(&self) -> [u16; 4] {
            [
                self.0 as u16,
                ((self.0 & 0x0000_0000_FFFF_0000) >> 16) as u16,
                ((self.0 & 0x0000_FFFF_0000_0000) >> 32) as u16,
                ((self.0 & 0xFFFF_0000_0000_0000) >> 48) as u16,
            ]
        }
        fn from_registers_sequential(value: &[u16; 4]) -> Self {
            Self(
                (value[0] as u64)
                    | (value[1] as u64) << 16
                    | (value[2] as u64) << 32
                    | (value[3] as u64) << 48,
            )
        }
    }
}
