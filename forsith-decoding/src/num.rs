use std::ops::{BitAnd, BitOr, BitXor, Shl, Shr, Mul, Sub, Div, Add};

fn read_exact_array<const N: usize, R: Read>(reader: &mut R) -> io::Result<[u8; N]> {
    let mut buf = [0u8; N];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

macro_rules! num_types {
    ($($num:ty),+) => {
        pub trait Num: Sized + Copy + Default + PartialEq + Eq + std::fmt::Debug + From<u8> + From<bool> $( + TryFrom<$num> ) +
        + BitAnd<Output=Self> + BitOr<Output=Self> + BitXor<Output=Self> + Shl<usize, Output=Self>
        + Shr<usize, Output=Self> + Add<Output=Self> + Sub<Output=Self> + Div<Output=Self> + Mul<Output=Self> {
            fn read_be<R: Read>(reader: &mut R) -> io::Result<Self> where Self: Sized;
            fn read_le<R: Read>(reader: &mut R) -> io::Result<Self> where Self: Sized;
            const BYTE_DEPTH: u8;
            const BIT_DEPTH: u8;
            const MAX: Self;
        }

        $(
        impl Num for $num {
            const BYTE_DEPTH: u8 = std::mem::size_of::<Self>() as u8;
            const BIT_DEPTH: u8 = Self::BYTE_DEPTH * 8;
            const MAX: Self = Self::MAX;
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

num_types!(usize, u64, u32, u16, u8);
