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

pub fn handle_special_case<DC: Channel, const DF: u8, SC: Channel, const SF: u8>(slice: &[u8], out: &mut OutputWriter<'_, DC, DF>, _padding: u8, _alpha_color: Option<(i64, i64, i64)>) -> bool {
    if (SC::MAX == DC::MAX) && (SC::MIN == DC::MIN) && out.stride == 0 {
        if DF == SF {
            out.buffer[out.index..out.index+slice.len()].copy_from_slice(slice);
            out.advance(slice.len()/SF as usize);
        } else {
            return false
        }

        return true;
    }

    false
}

// DC + DF = dest channel + format, SC + SF = source sample size + format
pub fn push_aligned_slice<DC: Channel, const DF: u8, SC: Channel, const SF: u8>(slice: &[u8], out: &mut OutputWriter<'_, DC, DF>, _padding: u8, alpha_color: Option<(i64, i64, i64)>)
where
    [(); SF as usize]:,
{
    if handle_special_case::<DC, DF, SC, SF>(slice, out, _padding, alpha_color) {return;}

    let bytespp = bytespp::<SC, SF>() as usize;
    for pixel in slice.chunks(bytespp) {
        let pixel_ptr = pixel.as_ptr() as *const SC::StorageType;

        #[cfg(debug_assertions)]
        if pixel_ptr.is_null() {panic!("pixel ptr null?")};

        convert_pixel::<SC, DF, SF>(pixel_ptr, alpha_color, |c| {
            let converted = convert_channel::<SC, DC>(c);

            out.push_channel(converted);
        });

        out.pushed_pixel();
    }
}

#[inline(always)]
fn convert_channel<SC: Channel, DC: Channel>(value: SC::StorageType) -> DC::StorageType {
    let value: i64 = value.to_be().into();

    // Normalize input to 0.0..1.0 integer space
    let normalized = (value - SC::MIN) as u64 * DC::MAX / (SC::MAX as i64 - SC::MIN) as u64;

    unsafe {DC::StorageType::try_from(normalized as i64 + DC::MIN).unwrap_unchecked()}
}

#[inline(always)]
fn read<T>(ptr: &mut *const T) -> T {
    unsafe {
        let r = if std::mem::align_of::<T>() == 1 {ptr.read()} else {ptr.read_unaligned()};

        (*ptr) = ptr.add(1); r
    }
}

#[inline(always)]
fn convert_pixel<C: Channel, const DF: u8, const SF: u8>(mut pixel: *const C::StorageType, alpha_color: Option<(i64, i64, i64)>, mut out: impl FnMut(C::StorageType)) {
    let color = if is_gray(DF) {
        let gray = if is_gray(SF) {read(&mut pixel)}
        else {
            let [r, g, b] = [read(&mut pixel), read(&mut pixel), read(&mut pixel)];
            let (r, g, b): (i64, i64, i64) = (r.into(), g.into(), b.into());
            unsafe {((299 * r + 587 * g  + 114 * b) / 1000).try_into().unwrap_unchecked()}
        };

        out(gray);

        (gray.into(), gray.into(), gray.into())
    } else {
        let rgb = if is_rgb(SF) {
            &[read(&mut pixel), read(&mut pixel), read(&mut pixel)]
        }
        else {
            let g = read(&mut pixel);
            &[g, g, g]
        };
        rgb.iter().for_each(|c| out(*c));

        (rgb[0].into(), rgb[1].into(), rgb[2].into())
    };

    if has_alpha(DF) {
        if has_alpha(SF) {out(read(&mut pixel))}
        else {
            if Some(color) == alpha_color {
                unsafe {out(C::StorageType::try_from(C::MIN).unwrap_unchecked())}
            } else {
                unsafe {out(C::StorageType::try_from(C::MAX).unwrap_unchecked())}
            }
        }
    }
}
