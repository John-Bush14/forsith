use std::{io::Read, ops::Range};
use derive_more::{Deref, IsVariant};
use crate::{DecodingError, buffers::CursorVec, parsing::{SegmentHeader, SegmentParser}};

const BLIND_LEN: usize = 1 << 10;

pub struct JpegParser<R: Read> {
    reader: R,
    marker: Marker,
    buffer: CursorVec<u8>,
    excess_bytes: usize,
}

impl <R: Read> JpegParser<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            marker: Marker { ty: MarkerType::Stuffing, len: 0 },
            buffer: CursorVec::<u8>::new(1 << 12),
            excess_bytes: 0,
        }
    }

    fn find_marker_indices(&self, start: usize, len: usize) -> Vec<usize> {
        self.buffer.get_ref()[start..start + len]
            .iter().enumerate()
            .filter_map(|(i, &b)| if b == 0xFF {Some(start + i)} else {None})
            .collect::<Vec<_>>()
    }

    fn remove_stuffing(&mut self, stuffing_indices: &[usize], end: usize) {
        for (stuffing_i, &i) in stuffing_indices.iter().enumerate().rev() {
            let end = *stuffing_indices.get(stuffing_i + 1).unwrap_or(&end);

            self.buffer.get_mut().copy_within((i + 2).min(end)..end, i);
        }

        self.buffer.unconsume(stuffing_indices.len() * 2);
    }

    fn ensure_no_border_markers(&mut self, start: usize, len: &mut usize) -> Result<(), crate::DecodingError> {
        while self.buffer.get_ref()[(start + len.saturating_sub(4)).max(start)..(start + *len)].iter().any(|x| x == &0xFF) {
            self.buffer.set_cursor(start + *len);
            *len += self.read_bytes(1)?;

             if self.buffer.cursor() - start == *len {break;}
        }

        if self.buffer.get_ref()[start + *len - 1] == 0xFF {Err(DecodingError::NoEOI)} else {Ok(())}
    }

    fn blind_read(&mut self,) -> Result<usize, crate::DecodingError> {
        let mut len = self.read_bytes(BLIND_LEN)?;
        if len == 0 {return Err(DecodingError::NoEOI)}
        self.ensure_no_border_markers(self.buffer.cursor(), &mut len)?; Ok(len)
    }

    fn handle_markers(&mut self, marker_indices: &Vec<usize>, data_ranges: &mut Vec<Range<usize>>) -> Result<Vec<usize>, crate::DecodingError> {
        let mut stuffing_indices: Vec<usize> = vec![];
        let mut stuffing_offset = 0;
        for i in marker_indices {
            self.buffer.set_cursor(*i);

            self.parse_header()?;
            if !self.marker.has_length_field() {self.excess_bytes -= 2;}

            if self.marker.is_stuffing() {
                stuffing_indices.push(*i);
                stuffing_offset += 2;
            } else {
                let i = i - stuffing_offset;

                let start = data_ranges.last().unwrap().end + 2;
                if start < i {
                    data_ranges.push(start..i);
                }

                if !(self.marker.is_rst() || self.marker.is_fill() || self.marker.is_stuffing()) {
                    break;
                }
            }
        }

        Ok(stuffing_indices)
    }
}

impl<R: Read> SegmentParser<R> for JpegParser<R> {
    type Header = Marker;
    type ExtraOut = Option<Vec<Range<usize>>>;

    fn context<'s, 'a, 'b, 'c>(&'s mut self) -> (&'a mut CursorVec<u8>, &'b mut R, &'c mut Self::Header)
        where 's: 'a, 's: 'b, 's: 'c
    {
        (&mut self.buffer, &mut self.reader, &mut self.marker)
    }

    fn handle_special_segment<F>(&mut self, out: &mut F) -> Result<(), crate::DecodingError>
        where F: FnMut(&Marker, &mut CursorVec<u8>, Option<Vec<Range<usize>>>) -> Result<(), crate::DecodingError>
    {
        self.clear_buffer();

        self.read_bytes_exact(self.marker.length())?;
        let mut start = self.buffer.cursor() + self.marker.length();
        #[allow(clippy::single_range_in_vec_init)]
        let mut data_ranges = vec![start - self.marker.length()..start];

        loop {
            self.buffer.set_cursor(start);
            let len = self.blind_read()?;

            let marker_indices = self.find_marker_indices(start, len);
            let stuffing_indices = self.handle_markers(&marker_indices, &mut data_ranges)?;
            self.remove_stuffing(&stuffing_indices, start + len);

            start += len - stuffing_indices.len()*2;

            if !(self.marker.is_rst() || self.marker.is_fill() || self.marker.is_stuffing()) {
                break;
            }
        }

        self.excess_bytes += start - self.buffer.cursor();

        let mut data = CursorVec::<u8>::new(0);
        std::mem::swap(&mut self.buffer, &mut data);
        *self.buffer.get_mut() = data.get_mut().drain(self.buffer.cursor()..).collect();

        let sos_marker = Marker {ty: MarkerType::Sos, len: u16::try_from(data_ranges.first().unwrap().len()).unwrap()};
        out(&sos_marker, &mut self.buffer, Some(data_ranges))
    }

    fn clear_buffer(&mut self) {
        let excess = self.buffer.cursor()..self.buffer.cursor() + self.excess_bytes;
        self.buffer.get_mut().copy_within(excess, 0);
        self.buffer.set_cursor(0);
    }

    fn read_bytes(&mut self, raw_len: usize) -> Result<usize, crate::DecodingError> {
        let len = raw_len.saturating_sub(self.excess_bytes);
        let excess_read = raw_len - len;

        self.buffer.consume(excess_read);
        let ret = Ok(self.read_bytes_default(len)? + excess_read);
        self.buffer.unconsume(excess_read);

        self.excess_bytes -= excess_read;

        ret
    }

    fn parse_header(&mut self) -> Result<(), crate::DecodingError> {
        self.marker = Marker::read(&mut *self.buffer)?;

        println!("Parsed marker: {:?}, length: {}", self.marker.ty, self.marker.len);

        if !self.marker.has_length_field() {self.excess_bytes += 2;}

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IsVariant)]
pub enum MarkerType {
    Stuffing,
    Fill,
    Soi,
    Sof(u8),
    Jpg,
    Dht,
    Dqt,
    Dri,
    Sos,
    Rst(u8),
    Dac,
    Dnl,
    Dhp,
    Exp,
    Jpgn(u8),
    Tem,
    App(u8),
    Com,
    Eoi,
}

impl MarkerType {
    #[allow(clippy::enum_glob_use)]
    const fn from_markercode(code: u8) -> Result<Self, DecodingError> {
        use MarkerType::*;
        Ok(match code {
            0x00 => Stuffing,
            0x01 => Tem,
            0xC4 => Dht,
            0xC8 => Jpg,
            0xCC => Dac,
            0xC0..=0xCF => Sof(code - 0xC0),
            0xD0..=0xD7 => Rst(code - 0xD0),
            0xD8 => Soi,
            0xD9 => Eoi,
            0xDA => Sos,
            0xDB => Dqt,
            0xDC => Dnl,
            0xDD => Dri,
            0xDE => Dhp,
            0xDF => Exp,
            0xE0..=0xEF => App(code - 0xE0),
            0xF0..=0xFD => Jpgn(code - 0xF0),
            0xFE => Com,
            0xFF => Fill, // Fill byte
            _ => return Err(DecodingError::InvalidMarkerCode(code)),
        })
    }

    #[allow(clippy::enum_glob_use)]
    const fn has_length_field(self) -> bool {
        use MarkerType::*;
        !matches!(self, Stuffing | Fill | Soi | Rst(_) | Eoi | Tem)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct Marker {
    #[deref]
    ty: MarkerType,
    len: u16
}

impl SegmentHeader for Marker {
    const MAX_SIZE: usize = 4;

    fn length(&self) -> usize {self.len as usize}

    fn size(&self) -> usize {if self.ty.has_length_field() {4} else {2}}

    fn read<R: Read>(reader: &mut R) -> Result<Self, crate::DecodingError> where Self: Sized {
        if reader.read_le::<u8>()? != 0xFF {return Err(crate::DecodingError::NoMarker);}

        let ty = MarkerType::from_markercode(reader.read_le::<u8>()?)?;
        let len = if ty.has_length_field() {reader.read_be::<u16>()?.checked_sub(2).ok_or(DecodingError::InvalidMarkerLen)?} else {0};

        println!("Read marker: {ty:?}, length: {len}");

        Ok(Self { ty, len })
    }

    fn is_final(&self) -> bool {self.is_eoi()}
    fn is_special(&self) -> bool {self.is_sos()}
}
