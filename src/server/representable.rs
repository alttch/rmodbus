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

/// The other side of [`RegisterRepresentable`], similar to how the
/// [`Into`] trait is the other side of Rust's [`From`] trait. This
/// trait is implemented on u16 buffers that can be converted to/from a
/// [`RegisterRepresentable`] type.
/// 
/// This trait is automatically implemented using a blanket impl. Do not
/// implement this trait manually.
pub trait RegisterBuffer<const N: usize, T: RegisterRepresentable<N>> {
    /// Convert this buffer into the represented type.
    fn to_represented(&self) -> T;
    /// Convert the represented type into an instance of this buffer.
    fn from_represented(value: &T) -> Self;
    /// Convert the represented type to its u16 registers representation,
    /// then copy that value into this buffer.
    fn copy_from_represented(&mut self, value: &T);
}

impl<const N: usize, T: RegisterRepresentable<N>> RegisterBuffer<N, T> for [u16; N] {
    fn to_represented(&self) -> T {
        T::from_registers_sequential(self)
    }
    fn from_represented(value: &T) -> Self {
        value.to_registers_sequential()
    }
    fn copy_from_represented(&mut self, value: &T) {
        self.copy_from_slice(value.to_registers_sequential().as_slice());
    }
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
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[cfg_attr(feature = "with_serde", derive(Deserialize, Serialize))]
    #[cfg_attr(feature = "with_bincode", derive(Decode, Encode))]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[cfg_attr(feature = "with_serde", derive(Deserialize, Serialize))]
    #[cfg_attr(feature = "with_bincode", derive(Decode, Encode))]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

    /// Tests specifically for the 4 representations provided
    #[cfg(test)]
    mod tests {
        #[allow(clippy::wildcard_imports)]
        use super::*;
        use super::super::RegisterBuffer;
        #[test]
        fn test_u32_big_small_endian() {
            let value: u32 = 0x1111_2222;
            let big_endian = U32BigEndian(value).to_registers_sequential();
            assert_eq!(<[u16; 2] as RegisterBuffer<2, U32BigEndian>>::to_represented(&big_endian), U32BigEndian(value));
            let little_endian = U32LittleEndian(value).to_registers_sequential();
            assert_eq!(<[u16; 2] as RegisterBuffer<2, U32LittleEndian>>::to_represented(&little_endian), U32LittleEndian(value));
            assert_eq!(big_endian[0], little_endian[1]);
            assert_eq!(big_endian[1], little_endian[0]);
        }
        #[test]
        fn test_u64_big_small_endian() {
            let value: u64 = 0x1111_2222_3333_4444;
            let big_endian = U64BigEndian(value).to_registers_sequential();
            assert_eq!(<[u16; 4] as RegisterBuffer<4, U64BigEndian>>::to_represented(&big_endian), U64BigEndian(value));
            let little_endian = U64LittleEndian(value).to_registers_sequential();
            assert_eq!(<[u16; 4] as RegisterBuffer<4, U64LittleEndian>>::to_represented(&little_endian), U64LittleEndian(value));
            assert_eq!(big_endian[0], little_endian[3]);
            assert_eq!(big_endian[1], little_endian[2]);
            assert_eq!(big_endian[2], little_endian[1]);
            assert_eq!(big_endian[3], little_endian[0]);
        }
    }
}
