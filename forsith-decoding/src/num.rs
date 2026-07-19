use std::ops::{BitAnd, BitOr, BitXor, Shl, Shr, Mul, Sub, Div, Add, Rem};

fn read_exact_array<const N: usize, R: Read>(reader: &mut R) -> io::Result<[u8; N]> {
    let mut buf = [0u8; N];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

macro_rules! int_types {
    ($($num:ty),+) => {
        pub trait Int: Sized + Copy + Default + PartialEq + Eq + std::fmt::Debug + From<bool> $( + TryFrom<$num> ) +
        + BitAnd<Output=Self> + BitOr<Output=Self> + BitXor<Output=Self> + Shl<usize, Output=Self> + Rem<Output=Self>
        + Shr<usize, Output=Self> + Add<Output=Self> + Sub<Output=Self> + Div<Output=Self> + Mul<Output=Self> {
            fn read_be<R: Read>(reader: &mut R) -> io::Result<Self> where Self: Sized;
            fn read_le<R: Read>(reader: &mut R) -> io::Result<Self> where Self: Sized;
            const BYTE_DEPTH: u8;
            const BIT_DEPTH: u8;
            const MAX: Self;
            const SIGNED: bool;
        }

        $(
        impl Int for $num {
            const BYTE_DEPTH: u8 = std::mem::size_of::<Self>() as u8;
            const BIT_DEPTH: u8 = Self::BYTE_DEPTH * 8;
            const MAX: Self = Self::MAX;
            const SIGNED: bool = <$num>::MIN != 0;
            fn read_be<R: Read>(reader: &mut R) -> io::Result<Self> {
                Ok(Self::from_be_bytes(read_exact_array::<{Self::BYTE_DEPTH as usize}, _>(reader)?))
            }
            fn read_le<R: Read>(reader: &mut R) -> io::Result<Self> {
                Ok(Self::from_le_bytes(read_exact_array::<{Self::BYTE_DEPTH as usize}, _>(reader)?))
            }
        }



        )+
    };
}

int_types!(usize, u64, u32, u16, u8, i8, i16, i32, i64, isize);
