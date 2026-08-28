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

pub fn unpack<const UPSAMPLE: bool>(slice: &[u8], bits: u8, padding: u8, callback: impl FnMut(&[u8])) {
    (match bits {
       1 => unpack_constant::<1, UPSAMPLE>,
       2 => unpack_constant::<2, UPSAMPLE>,
       4 => unpack_constant::<4, UPSAMPLE>,
       _ => unreachable!()
    })(slice, padding, callback);
}

#[inline(always)]
pub fn unpack_constant<const BITS: u8, const UPSAMPLE: bool>(slice: &[u8], padding: u8, mut callback: impl FnMut(&[u8])) {
    let mut i = 0; loop {
        let b = slice[i] as usize;

        let bytes = if UPSAMPLE { match BITS {
            1 => {UPSAMPLE_1BIT[b].as_slice()},
            2 => {UPSAMPLE_2BIT[b].as_slice()},
            4 => {UPSAMPLE_4BIT[b].as_slice()},
            _ => unreachable!()
        } } else { match BITS {
            1 => {UNPACK_1BIT[b].as_slice()},
            2 => {UNPACK_2BIT[b].as_slice()},
            4 => {UNPACK_4BIT[b].as_slice()},
            _ => unreachable!()
        } };

        if i == slice.len() - 1 {
            callback(&bytes[..bytes.len() - (padding/BITS) as usize]);

            break;
        }

        callback(bytes);

        i += 1;
    }
}

#[allow(clippy::cast_possible_truncation)]
const fn make_unpack_lut<const BITS: usize, const SAMPLES: usize, const UPSAMPLE: bool>() -> [[u8; SAMPLES]; 256] {
    let mut lut = [[0u8; SAMPLES]; 256];

    let mut byte = 0;
    while byte < 256 {
        let mut i = 0;
        while i < SAMPLES {
            let shift = 8 - BITS * (i + 1);
            let sample = (byte >> shift) & ((1 << BITS) - 1);

            // Expand to 8-bit range
            lut[byte][i] = if UPSAMPLE {(sample * 255 / ((1 << BITS) - 1)) as u8} else {sample as u8};

            i += 1;
        }
        byte += 1;
    }

    lut
}

pub const UPSAMPLE_1BIT: [[u8; 8]; 256] = make_unpack_lut::<1, 8, true>();
pub const UPSAMPLE_2BIT: [[u8; 4]; 256] = make_unpack_lut::<2, 4, true>();
pub const UPSAMPLE_4BIT: [[u8; 2]; 256] = make_unpack_lut::<4, 2, true>();

pub const UNPACK_1BIT: [[u8; 8]; 256] = make_unpack_lut::<1, 8, false>();
pub const UNPACK_2BIT: [[u8; 4]; 256] = make_unpack_lut::<2, 4, false>();
pub const UNPACK_4BIT: [[u8; 2]; 256] = make_unpack_lut::<4, 2, false>();
