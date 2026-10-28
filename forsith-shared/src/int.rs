use core::{
    fmt::{Debug, Display}, ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign},
};

pub trait ArithmeticOperations:
    PartialEq
    + PartialOrd
    + From<bool>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
    + Mul<Output = Self>
    + AddAssign
    + SubAssign
    + DivAssign
    + RemAssign
    + MulAssign
{
}

pub trait BitOperations:
    From<bool>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + BitAndAssign
    + BitOrAssign
    + BitXorAssign
    + ShlAssign
    + ShrAssign
{
}

pub trait UInt: Int + From<u8> {}
pub trait SInt: Int + From<i8> {}

macro_rules! num_types {
    (unsigned: [$($unum:ty),+], signed: [$($snum:ty),+]) => {
        pub trait Int:
            ArithmeticOperations
            + BitOperations
            + Debug
            + Display
            + Copy
            $(+ TryFrom<$snum> + TryInto<$snum>)+
            $(+ TryFrom<$unum> + TryInto<$unum>)+
        {
            const BYTE_DEPTH: u8;
            const BIT_DEPTH: u8;
            const MAX: u64;
            const MIN: i64;
            const SIGNED: bool;

            fn as_usize(self) -> usize {
                self.try_into().unwrap_or_else(|_| panic!("Value {} cannot be converted to usize", self))
            }

            fn as_u64(self) -> u64 {
                self.try_into().unwrap_or_else(|_| panic!("Value {} cannot be converted to u64", self))
            }
        }

        $(
            impl ArithmeticOperations for $unum {}
            impl BitOperations for $unum {}
            #[allow(clippy::cast_possible_truncation)]
            impl Int for $unum {
                const BYTE_DEPTH: u8 = core::mem::size_of::<$unum>() as u8;
                const BIT_DEPTH: u8 = Self::BYTE_DEPTH * 8;
                const MAX: u64 = <$unum>::MAX as u64;
                const MIN: i64 = 0;
                const SIGNED: bool = false;
            }
            impl UInt for $unum {}
        )+
        $(
            impl ArithmeticOperations for $snum {}
            impl BitOperations for $snum {}
            #[allow(clippy::cast_possible_truncation)]
            impl Int for $snum {
                const BYTE_DEPTH: u8 = core::mem::size_of::<$snum>() as u8;
                const BIT_DEPTH: u8 = Self::BYTE_DEPTH * 8;
                const MAX: u64 = <$snum>::MAX as u64;
                const MIN: i64 = <$snum>::MIN as i64;
                const SIGNED: bool = true;
            }
            impl SInt for $snum {}
        )+
    };

}

num_types!(
    unsigned: [u8, u16, u32, u64, usize],
    signed: [i8, i16, i32, i64, isize]
);
