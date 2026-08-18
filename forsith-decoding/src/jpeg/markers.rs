use std::io::BufRead;

use derive_more::IsVariant;
use crate::{Channel, DecodingError, JpegDecoder, jpeg::parser::Marker, parsing::SegmentHeader};


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
    pub fn update_decoder<C: Channel, const F: u8>(decoder: &mut JpegDecoder<'_, C, F>, marker: Marker, data: impl BufRead) -> Result<(), DecodingError> {
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
