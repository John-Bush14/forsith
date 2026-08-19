use std::{io::BufRead, simd::Simd};

use derive_more::IsVariant;
use crate::{Channel, DecodingError, ImageDecoder, JpegDecoder, jpeg::{idct::IdctTable, parser::Marker}, parsing::SegmentHeader};
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

pub struct ScanMetadata;
impl ScanMetadata {
    pub fn update_decoder<C: Channel, const F: u8>(decoder: &mut JpegDecoder<'_, C, F>, marker: Marker, data: impl BufRead) -> Result<(), DecodingError> {
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

#[derive(Debug)]
pub struct FrameHeader {
    precision: u8,
    dimensions: (u16, u16),
    components: Vec<FrameComponent>,
}
impl FrameHeader {
    pub fn update_decoder<C: Channel, const F: u8>(decoder: &mut JpegDecoder<'_, C, F>, marker: Marker, data: &mut impl BufRead) -> Result<(), DecodingError> {
        let MarkerType::Sof(id) = *marker else {panic!("FrameHeader::update_decoder called with non-SOF marker")};

        let huffman = id <= 7; let arithmetic = !huffman;

        let differential = id & 4 != 0;

        let baseline = matches!(id % 4, 0);
        let sequential = matches!(id % 4, 1);
        let progressive = matches!(id % 4, 2);
        let lossless = matches!(id % 4, 3);

        let precision = data.read_be::<u8>()?;
        let dimensions = (data.read_be::<u16>()?, data.read_be::<u16>()?);
        let component_count = data.read_be::<u8>()?;

        if
            component_count != u8::try_from((marker.length() - 6) / 3).unwrap()
            || !(2..=16).contains(&precision)
            || progressive && component_count > 4
            || component_count == 0
            || !lossless && !matches!(precision, 8 | 12)
            || baseline && precision != 8
        {
            return Err(DecodingError::InvalidMarker(*marker));
        }

        let header = Self {
            precision,
            dimensions,
            components: (0..component_count).map(|_| FrameComponent::read(data)).collect::<Result<Vec<_>, _>>()?,
        };

        if header.components.iter().any(|c|
            !(1..=4).contains(&c.sampling_factors.0) || !(1..=4).contains(&c.sampling_factors.1)
            || c.quantization_table > 3
            || lossless && c.quantization_table != 0
        ) {
            return Err(DecodingError::InvalidMarker(*marker));
        }

        decoder.frames.push(header); Ok(())
    }

    #[allow(dead_code)]
    pub const fn precision(&self) -> u8 {self.precision}
    #[allow(dead_code)]
    pub const fn dimensions(&self) -> (u16, u16) {self.dimensions}
    #[allow(dead_code)]
    pub fn components(&self) -> &[FrameComponent] {&self.components}
}

pub struct QuantizationTables;
impl QuantizationTables {
    pub fn update_decoder<C: Channel, const F: u8>(decoder: &mut JpegDecoder<'_, C, F>, marker: Marker, data: &mut impl BufRead) -> Result<(), DecodingError> {
        let mut remaining = marker.length();

        let mut quant_table = [Simd::splat(0); 8];

        while remaining > 0 {
            let table_info = data.read_be::<u8>()?;
            let id = table_info & 0x0F;
            let precision = match table_info >> 4 {
                0 => 8,
                1 => 16,
                _ => return Err(DecodingError::InvalidMarker(*marker)),
            };
            assert!(precision == 8, "Currently only 8-bit quantization tables are supported");
            remaining = remaining.checked_sub(1 + 64 * (precision / 8)).ok_or(DecodingError::InvalidMarker(*marker))?;

            let values = &data.fill_buf()?[..64];
            #[unroll]
            for i in 0..64 {
                let (j, k) = DEZIGZAG_MATRIX_TABLE[i];
                quant_table[j][k] = i32::from(values[i]);
            }
            data.consume(64);

            let idct_table = IdctTable::load(quant_table);

            // todo!("Store table in decoder");
        }

        Ok(())
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
