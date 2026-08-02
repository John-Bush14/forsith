use crate::{Channel, CursorVec, Int, OutputWriter, bytespp, has_alpha, is_gray, is_rgb, unpack_constant};

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

#[allow(type_alias_bounds)]
pub type OutputConverter<C: Channel, const F: u8> = fn(&[u8], &mut OutputWriter<C, F>, u8, Option<(i64, i64, i64)>);

pub fn get_out_writer_func<C: Channel, const F: u8>(sample_size: u8, format: u8, signed: bool) -> OutputConverter<C, F>
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

pub fn push_packed_slice<DC: Channel, const DF: u8, const SC: u8, const SF: u8>(slice: &[u8], out: &mut OutputWriter<'_, DC, DF>, padding: u8, alpha_color: Option<(i64, i64, i64)>)
where
    [(); SF as usize]:,
{
    let mut pixels = CursorVec::<u8>::new(match SF {3 => 24,  _ => 8});

    unpack_constant::<SC, true>(slice, padding, |bytes| {
        pixels.push_slice(bytes);

        if pixels.is_full() {
            push_aligned_slice::<DC, DF, u8, SF>(pixels.as_slice(), out, 0, alpha_color);
            pixels.clear();
    }
    });

    if !pixels.is_empty() {
        push_aligned_slice::<DC, DF, u8, SF>(pixels.as_slice(), out, 0, alpha_color);
    }
}

// DC + DF = dest channel + format, SC + SF = source sample size + format
pub fn push_aligned_slice<DC: Channel, const DF: u8, SC: Channel, const SF: u8>(slice: &[u8], out: &mut OutputWriter<'_, DC, DF>, _padding: u8, alpha_color: Option<(i64, i64, i64)>)
where
    [(); SF as usize]:,
{
    let bytespp = bytespp::<SC, SF>() as usize;

    let mut samples = SC::StorageType::iterate_bytes_be(slice);

    for _ in 0..(slice.len() / bytespp) {
        convert_pixel::<SC, DF, SF>(&mut samples, alpha_color, |c| {
            let converted = convert_channel::<SC, DC>(c);

            out.push_channel(converted);
        });

        out.pushed_pixel();
    }
}

#[inline(always)]
fn convert_channel<SC: Channel, DC: Channel>(value: SC::StorageType) -> DC::StorageType {
    let value: i64 = value.into();

    // Normalize input to 0.0..1.0 integer space
    let normalized = (value - SC::MIN) as u64 * DC::MAX / (SC::MAX as i64 - SC::MIN) as u64;

    DC::StorageType::try_from(normalized as i64 + DC::MIN).unwrap()
}

#[inline(always)]
fn convert_pixel<C: Channel, const DF: u8, const SF: u8>(samples: &mut impl Iterator<Item=C::StorageType>, alpha_color: Option<(i64, i64, i64)>, mut out: impl FnMut(C::StorageType)) {
    let color = if is_gray(DF) {
        let gray = if is_gray(SF) {samples.next().unwrap()}
        else {
            let [r, g, b] = [samples.next().unwrap(), samples.next().unwrap(), samples.next().unwrap()];
            let (r, g, b): (i64, i64, i64) = (r.into(), g.into(), b.into());
            ((299 * r + 587 * g  + 114 * b) / 1000).try_into().unwrap()
        };

        out(gray);

        (gray.into(), gray.into(), gray.into())
    } else {
        let rgb = if is_rgb(SF) {
            &[samples.next().unwrap(), samples.next().unwrap(), samples.next().unwrap()]
        }
        else {
            let g = samples.next().unwrap();
            &[g, g, g]
        };
        rgb.iter().for_each(|c| out(*c));

        (rgb[0].into(), rgb[1].into(), rgb[2].into())
    };

    if has_alpha(DF) {
        if has_alpha(SF) {out(samples.next().unwrap())}
        else {
            if Some(color) == alpha_color {
                out(C::StorageType::try_from(C::MIN).unwrap())
            } else {
                out(C::StorageType::try_from(C::MAX).unwrap())
            }
        }
    }
}
