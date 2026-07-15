use std::ops::{BitAnd, BitOr, BitXor, Shl, Shr};

fn read_exact_array<const N: usize, R: Read>(reader: &mut R) -> io::Result<[u8; N]> {
    let mut buf = [0u8; N];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

pub trait Num: Sized + Copy + Default + PartialEq
+ Eq + std::fmt::Debug + TryFrom<u32> + From<u8> + TryFrom<u16> + TryFrom<usize>
+ BitAnd<Output=Self> + BitOr<Output=Self> + BitXor<Output=Self> + Shl<usize, Output=Self>
+ Shr<usize, Output=Self> + std::ops::Add<Output=Self> + std::ops::Sub<Output=Self>
+ std::ops::Div<Output=Self> + std::ops::Mul<Output=Self> {
    fn read_be<R: Read>(reader: &mut R) -> io::Result<Self> where Self: Sized;
    fn read_le<R: Read>(reader: &mut R) -> io::Result<Self> where Self: Sized;
    const BIT_DEPTH: u8;
    const MAX: Self;
}
impl Num for u32 {
    fn read_be<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u32::from_be_bytes(read_exact_array::<4, _>(reader)?))
    }
    fn read_le<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u32::from_le_bytes(read_exact_array::<4, _>(reader)?))
    }
    const BIT_DEPTH: u8 = std::mem::size_of::<Self>() as u8 * 8;
    const MAX: Self = Self::MAX;
}
impl Num for u16 {
    fn read_be<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u16::from_be_bytes(read_exact_array::<2, _>(reader)?))
    }
    fn read_le<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u16::from_le_bytes(read_exact_array::<2, _>(reader)?))
    }
    const BIT_DEPTH: u8 = std::mem::size_of::<Self>() as u8 * 8;
    const MAX: Self = Self::MAX;
}
impl Num for u8 {
    fn read_be<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u8::from_be_bytes(read_exact_array::<1, _>(reader)?))
    }
    fn read_le<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u8::from_le_bytes(read_exact_array::<1, _>(reader)?))
    }
    const BIT_DEPTH: u8 = std::mem::size_of::<Self>() as u8 * 8;
    const MAX: Self = Self::MAX;
}
impl Num for usize {
    fn read_be<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(usize::from_be_bytes(read_exact_array::<{std::mem::size_of::<Self>()}, R>(reader)?))
    }
    fn read_le<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(usize::from_le_bytes(read_exact_array::<{std::mem::size_of::<Self>()}, R>(reader)?))
    }
    const BIT_DEPTH: u8 = std::mem::size_of::<Self>() as u8 * 8;
    const MAX: Self = Self::MAX;
}
