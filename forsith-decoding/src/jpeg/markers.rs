use std::io::BufRead;

use derive_more::IsVariant;
use crate::{Channel, DecodingError, JpegDecoder, jpeg::parser::Marker};


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

pub struct FrameHeader;
impl FrameHeader {
    pub fn update_decoder<C: Channel, const F: u8>(decoder: &mut JpegDecoder<'_, C, F>, marker: Marker, data: impl BufRead) -> Result<(), DecodingError> {
        todo!();
    }
}

pub struct QuantizationTables;
impl QuantizationTables {
    pub fn update_decoder<C: Channel, const F: u8>(decoder: &mut JpegDecoder<'_, C, F>, marker: Marker, data: impl BufRead) -> Result<(), DecodingError> {
        todo!();
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
