use std::ops::{BitAnd, BitOr, BitXor, Shl, Shr, Mul, Sub, Div, Add, Rem};

macro_rules! int_types {
    ($($num:ty),+) => {
        pub trait Int: Sized + Copy + Default + PartialEq + Eq + std::fmt::Debug + From<bool> $( + TryFrom<$num> ) +
        + BitAnd<Output=Self> + BitOr<Output=Self> + BitXor<Output=Self> + Shl<usize, Output=Self> + Rem<Output=Self>
        + Shr<usize, Output=Self> + Add<Output=Self> + Sub<Output=Self> + Div<Output=Self> + Mul<Output=Self> + Into<i64>
        + TryFrom<i64> + TryFrom<u64> + TryInto<u64> + TryInto<i64>
        {
            fn iterate_bytes_be(slice: &[u8]) -> impl Iterator<Item=Self>;
            const BYTE_DEPTH: u8;
            const BIT_DEPTH: u8;
            const MAX: u64;
            const MIN: i64;
            const SIGNED: bool;
        }

        $(
        #[allow(clippy::cast_possible_truncation)]
        impl Int for $num {
            const BYTE_DEPTH: u8 = std::mem::size_of::<Self>() as u8;
            const BIT_DEPTH: u8 = Self::BYTE_DEPTH * 8;
            const MAX: u64 = Self::MAX as _;
            const MIN: i64 = Self::MIN as _;
            const SIGNED: bool = <$num>::MIN != 0;
            #[inline(always)]
            fn iterate_bytes_be(slice: &[u8]) -> impl Iterator<Item=Self> {
                slice.chunks_exact(Self::BYTE_DEPTH as usize).map(|b| Self::from_be_bytes(b.try_into().unwrap()))
            }
        }
        )+
    };
}

int_types!(u32, u16, u8, i8, i16, i32);
