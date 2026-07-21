use crate::{Channel, CursorVec, Int, OutputWriter, bitspp, bytespp};

macro_rules! aligned {
    ($t:ty, $format:ident) => {
        match $format {
            1 => push_aligned_slice::<C, F, $t, 1>,
            2 => push_aligned_slice::<C, F, $t, 2>,
            3 => push_aligned_slice::<C, F, $t, 3>,
            4 => push_aligned_slice::<C, F, $t, 4>,
            _ => unreachable!(),
        }
    };
}

macro_rules! packed {
    ($d:expr, $format:ident) => {
        match $format {
            1 => push_packed_slice::<C, F, $d, 1>,
            2 => push_packed_slice::<C, F, $d, 2>,
            3 => push_packed_slice::<C, F, $d, 3>,
            4 => push_packed_slice::<C, F, $d, 4>,
            _ => unreachable!()
        }
    };
}

pub type OutputConverter = fn(&[u8], &mut OutputWriter, u8, Option<(i64, i64, i64)>);

pub fn get_out_writer_func<C: Channel, const F: u8>(sample_size: u8, format: u8, signed: bool) -> OutputConverter
{
    match sample_size {
        1 => packed!(1, format),
        2 => packed!(2, format),
        4 => packed!(4, format),
        8 => match signed {false => aligned!(u8, format), true => aligned!(i8, format)}
        16 => match signed {false => aligned!(u16, format), true => aligned!(i16, format)}
        32 => match signed {false => aligned!(u32, format), true => aligned!(i32, format)}
        _ => todo!()
    }
}

pub fn push_packed_slice<DC: Channel, const DF: u8, const SC: u8, const SF: u8>(slice: &[u8], out: &mut OutputWriter, padding: u8, alpha_color: Option<(i64, i64, i64)>)
where
    [(); SF as usize]:,
{
    let padding_pixels = padding/SF;

    let mut pixels = CursorVec::<u8>::new(match SF {3 => 24,  _ => 8});
    for i in 0..slice.len() {
        let b = slice[i] as usize;

        let bytes = match SC {
            1 => {UPSAMPLE_1BIT[b].as_slice()},
            2 => {UPSAMPLE_2BIT[b].as_slice()},
            4 => {UPSAMPLE_4BIT[b].as_slice()},
            _ => unreachable!()
        };

        if i == slice.len() - 1 {
            pixels.push_slice(&bytes[..bytes.len() - padding_pixels as usize]);

            push_aligned_slice::<DC, DF, u8, SF>(pixels.as_slice(), out, 0, alpha_color);

            return
        }

        pixels.push_slice(bytes);

        if pixels.is_full() {
            push_aligned_slice::<DC, DF, u8, SF>(pixels.as_slice(), out, 0, alpha_color);
            pixels.clear();
        }
    }
}

// DC + DF = dest channel + format, SC + SF = source sample size + format
pub fn push_aligned_slice<DC: Channel, const DF: u8, SC: Channel, const SF: u8>(slice: &[u8], out: &mut OutputWriter, _padding: u8, alpha_color: Option<(i64, i64, i64)>)
where
    [(); SF as usize]:,
{
    let bytespp = bytespp::<SC, SF>() as usize;
    for i in (0..slice.len()).step_by(bytespp) {
        let pixel_ptr = unsafe {(slice.get_unchecked(i..i + bytespp).as_ptr() as *const [SC::StorageType; SF as usize])};

        #[cfg(debug_assertions)]
        if pixel_ptr.is_null() {panic!("pixel ptr null?")};

        let pixel = unsafe {&*pixel_ptr};

        convert_pixel::<SC, DF, SF>(pixel, alpha_color, |c| {
            let converted = convert_channel::<SC, DC>(c);

            out.push_channel::<DC>(converted);
        });
    }
}

fn convert_channel<SC: Channel, DC: Channel>(value: SC::StorageType) -> DC::StorageType {
    let value: i64 = value.to_be().into();

    // Normalize input to 0.0..1.0 integer space
    let normalized = (value - SC::MIN) as u64 * DC::MAX / (SC::MAX as i64 - SC::MIN) as u64;

    unsafe {DC::StorageType::try_from(normalized as i64 + DC::MIN).unwrap_unchecked()}
}

fn convert_pixel<C: Channel, const DF: u8, const SF: u8>(pixel: &[C::StorageType; SF as usize], alpha_color: Option<(i64, i64, i64)>, mut out: impl FnMut(C::StorageType)) {
    let mut i = 0;

    // grayscale
    let color = if DF <= 2 {
        let gray = if SF <= 2 {i += 1; pixel[0]}
        else {
            i += 3;
            let [r, g, b] = pixel[0..2] else {unreachable!()};
            let (r, g, b): (i64, i64, i64) = (r.into(), g.into(), b.into());
            unsafe {((299 * r + 587 * g  + 114 * b) / 1000).try_into().unwrap_unchecked()}
        };

        out(gray);

        (gray.into(), gray.into(), gray.into())
    }

    // rgb
    else {
        let rgb = if SF > 2 {
            i += 3;
            &pixel[0..3]
        }
        else {
            let g = pixel[0];i += 1;
            &[g, g, g]
        };
        rgb.iter().for_each(|c| out(*c));

        (rgb[0].into(), rgb[1].into(), rgb[2].into())
    };

    // has alpha
    if DF.is_multiple_of(2) {
        if SF.is_multiple_of(2) {out(pixel[i])}
        else {
            if Some(color) == alpha_color {
                unsafe {out(C::StorageType::try_from(C::MIN).unwrap_unchecked())}
            } else {
                unsafe {out(C::StorageType::try_from(C::MAX).unwrap_unchecked())}
            }
        }
    }
}

const fn make_upsample_lut<const BITS: usize, const SAMPLES: usize>() -> [[u8; SAMPLES]; 256] {
    let mut lut = [[0u8; SAMPLES]; 256];

    let mut byte = 0;
    while byte < 256 {
        let mut i = 0;
        while i < SAMPLES {
            let shift = 8 - BITS * (i + 1);
            let sample = (byte >> shift) & ((1 << BITS) - 1);

            // Expand to 8-bit range
            lut[byte][i] = (sample * 255 / ((1 << BITS) - 1)) as u8;

            i += 1;
        }
        byte += 1;
    }

    lut
}

pub const UPSAMPLE_1BIT: [[u8; 8]; 256] = make_upsample_lut::<1, 8>();
pub const UPSAMPLE_2BIT: [[u8; 4]; 256] = make_upsample_lut::<2, 4>();
pub const UPSAMPLE_4BIT: [[u8; 2]; 256] = make_upsample_lut::<4, 2>();
