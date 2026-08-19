use std::{io::BufRead, ops::Range, simd::Simd};

use derive_more::IsVariant;
use crate::{Channel, DecodingError, ImageDecoder, JpegDecoder, buffers::CursorVec, jpeg::{DecodeOp, idct::IdctTable, parser::Marker}, parsing::SegmentHeader};
use const_for::const_for;

const DEZIGZAG_TABLE: [usize; 64] = [
    0, 1, 5, 6,14,15,27,28,
    2, 4, 7,13,16,26,29,42,
    3, 8,12,17,25,30,41,43,
    9,11,18,24,31,40,44,53,
   10,19,23,32,39,45,52,54,
   20,22,33,38,46,51,55,60,
   21,34,37,47,50,56,59,61,
   35,36,48,49,57,58,62,63
];

const DEZIGZAG_MATRIX_TABLE: [(usize, usize); 64] = {
    let mut table = [(0usize, 0usize); 64];
    const_for!(i in 0..64 => {
        let full = DEZIGZAG_TABLE[i];
        table[i] = (full.div_euclid(8), full % 8);
    });
    table
};

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
    pub const fn from_markercode(code: u8) -> Result<Self, DecodingError> {
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
    pub const fn has_length_field(self) -> bool {
        use MarkerType::*;
        !matches!(self, Stuffing | Fill | Soi | Rst(_) | Eoi | Tem)
    }
}

#[derive(Debug)]
pub struct Scan;
impl Scan {
    pub fn update_decoder<C: Channel, const F: u8>(decoder: &mut JpegDecoder<'_, C, F>, marker: Marker, data: CursorVec<u8>, data_ranges: CursorVec<Range<usize>>) -> Result<(), DecodingError> {
        todo!();
    }
}

#[derive(Debug)]
pub struct FrameComponent {
    pub id: u8,
    pub sampling_factors: (u8, u8),
    pub quantization_table: u8,
}
impl FrameComponent {
    pub fn read<R: BufRead>(reader: &mut R) -> Result<Self, DecodingError> {
        let id = reader.read_be::<u8>()?;
        let sampling_factors = reader.read_be::<u8>()?;
        let quantization_table = reader.read_be::<u8>()?;

        Ok(Self {
            id,
            sampling_factors: (sampling_factors >> 4, sampling_factors & 0x0F),
            quantization_table,
        })
    }
}

#[derive(Debug, IsVariant, Clone, Copy)]
pub enum EntropyCoding {
    Huffman,
    Arithmetic,
}

#[derive(Debug, IsVariant, Clone, Copy)]
pub enum FrameType {
    Baseline,
    Sequential,
    Progressive,
    Lossless,
}

#[derive(Debug)]
pub struct FrameHeader {
    precision: u8,
    dimensions: (u16, u16),
    components: Vec<FrameComponent>,
    entropy_coding: EntropyCoding,
    frame_type: FrameType,
    differential: bool,
}
impl FrameHeader {
    pub fn update_decoder<C: Channel, const F: u8>(decoder: &mut JpegDecoder<'_, C, F>, marker: Marker, data: &mut impl BufRead) -> Result<(), DecodingError> {
        let MarkerType::Sof(id) = *marker else {panic!("FrameHeader::update_decoder called with non-SOF marker")};

        let entropy_coding = if id <= 7 {EntropyCoding::Huffman} else {EntropyCoding::Arithmetic};
        let differential = id & 4 != 0;
        let frame_type = match id % 4 {
            0 => FrameType::Baseline,
            1 => FrameType::Sequential,
            2 => FrameType::Progressive,
            3 => FrameType::Lossless,
            _ => unreachable!()
        };

        let precision = data.read_be::<u8>()?;
        let dimensions = (data.read_be::<u16>()?, data.read_be::<u16>()?);
        let component_count = data.read_be::<u8>()?;

        Self::validate_metadata(marker, precision, frame_type, component_count)?;

        let header = Self {
            precision,
            dimensions,
            components: (0..component_count).map(|_| FrameComponent::read(data)).collect::<Result<Vec<_>, _>>()?,
            entropy_coding,
            frame_type,
            differential,
        };

        if !header.valid_components() {return Err(DecodingError::InvalidMarker(*marker));}

        decoder.push_frame(header);

        Ok(())
    }

    fn validate_metadata(marker: Marker, precision: u8, frame_type: FrameType, component_count: u8) -> Result<(), DecodingError> {
        if
            component_count != u8::try_from((marker.length() - 6) / 3).unwrap()
            || !(2..=16).contains(&precision)
            || frame_type.is_progressive() && component_count > 4
            || component_count == 0
            || !frame_type.is_lossless() && !matches!(precision, 8 | 12)
            || frame_type.is_baseline() && precision != 8
        {
            Err(DecodingError::InvalidMarker(*marker))
        } else {
            Ok(())
        }
    }

    fn valid_components(&self) -> bool {
        !self.components.iter().any(|c|
            !(1..=4).contains(&c.sampling_factors.0) || !(1..=4).contains(&c.sampling_factors.1)
            || c.quantization_table > 3
            || self.frame_type.is_lossless() && c.quantization_table != 0
        )
    }

    #[allow(dead_code)]
    pub const fn precision(&self) -> u8 {self.precision}
    #[allow(dead_code)]
    pub const fn dimensions(&self) -> (u16, u16) {self.dimensions}
    #[allow(dead_code)]
    pub fn components(&self) -> &[FrameComponent] {&self.components}
    #[allow(dead_code)]
    pub const fn entropy_coding(&self) -> EntropyCoding {self.entropy_coding}
    #[allow(dead_code)]
    pub const fn frame_type(&self) -> FrameType {self.frame_type}
    #[allow(dead_code)]
    pub const fn differential(&self) -> bool {self.differential}
}

pub struct QuantizationTables;
impl QuantizationTables {
    pub fn update_decoder<C: Channel, const F: u8>(decoder: &mut JpegDecoder<'_, C, F>, marker: Marker, data: &mut impl BufRead) -> Result<(), DecodingError> {
        let mut remaining = marker.length();

        let mut quant_table = [Simd::splat(0); 8];

        while remaining > 0 {
            let (id, precision) = Self::read_table_info(data)?;
            remaining = remaining.checked_sub(1 + 64 * (precision as usize / 8)).ok_or(DecodingError::InvalidMarker(*marker))?;

            Self::read_quant_table(data, precision, &mut quant_table)?;

            let idct_table = IdctTable::load(quant_table);
            decoder.decode_timeline.push(DecodeOp::SetQuantizationTable(id as _, Box::new(idct_table)));
        }

        Ok(())
    }

    fn read_quant_table<R: BufRead>(reader: &mut R, _precision: u8, table: &mut [Simd<i32, 8>; 8]) -> Result<(), DecodingError> {
        let values = &reader.fill_buf()?[..64];
        #[unroll]
        for i in 0..64 {
            let (j, k) = DEZIGZAG_MATRIX_TABLE[i];
            table[j][k] = i32::from(values[i]);
        }
        reader.consume(64);
        Ok(())
    }

    fn read_table_info<R: BufRead>(reader: &mut R) -> Result<(u8, u8), DecodingError> {
        let table_info = reader.read_be::<u8>()?;
        let id = table_info & 0x0F;
        let precision = match table_info >> 4 {
            0 => 8,
            1 => 16,
            _ => return Err(DecodingError::InvalidMarker(MarkerType::Dqt)),
        };
        assert!(precision == 8, "Currently only 8-bit quantization tables are supported");
        Ok((id, precision))
    }
}

pub struct HuffmanTables;
impl HuffmanTables {
    pub fn update_decoder<C: Channel, const F: u8>(decoder: &mut JpegDecoder<'_, C, F>, marker: Marker, data: impl BufRead) -> Result<(), DecodingError> {
        todo!();
    }
}

pub struct RestartInterval;
impl RestartInterval {
    pub fn update_decoder<C: Channel, const F: u8>(decoder: &mut JpegDecoder<'_, C, F>, marker: Marker, data: impl BufRead) -> Result<(), DecodingError> {
        todo!();
    }
}
