use std::{io::Read, ops::Range};
use crate::{Channel, DecodingError, ImageDecoder, PixelFormat, buffers::CursorVec, decompression::HuffmanTree, jpeg::{idct::IdctTable, markers::{FrameHeader, HuffmanTables, MarkerType, QuantizationTables, RestartInterval, Scan}, parser::Marker}, parsing::{SegmentHeader, SegmentParser}};

pub mod markers;

mod idct;

mod parser;
use parser::JpegParser;

const JPEG_HEADER: [u8; 2] = [0xFF, 0xD8];

#[derive(Debug)]
pub struct JpegDecoder<'a, C: Channel, const F: u8> {
    phantom: std::marker::PhantomData<&'a C>,
    frames: Vec<FrameHeader>,
    decode_timeline: Vec<DecodeOp>,
}

#[derive(Debug)]
enum DecodeOp {
    SetFrame(usize),
    SetQuantizationTable(usize, Box<IdctTable>),
    Scan(Box<Scan>)
}

impl<'a, C: Channel, const F: u8> ImageDecoder<'a, C, F> for JpegDecoder<'a, C, F> {
    fn open_validated<R: Read>(mut reader: R) -> Result<Self, DecodingError> where Self: Sized {
        check_header(&mut reader)?;

        let mut parser = JpegParser::new(reader);

        let _ = parser.parse_first_chunk()?;

        let mut decoder = Self {
            phantom: std::marker::PhantomData,
            frames: Vec::new(),
            decode_timeline: Vec::new(),
        };

        parser.parse_chunks(|header, data, data_ranges| decoder.update_with_marker(*header, data, data_ranges))?;

        Ok(decoder)
    }

    fn read(&mut self, _buf: &mut [<C as Channel>::StorageType]) -> Result<usize, DecodingError> {
        todo!()
    }

    fn image_dimensions(&self) -> (usize, usize) {
        todo!()
    }

    fn min_buf_size(&self) -> usize {
        todo!()
    }

    fn source_bit_depth(&self) -> u8 {
        self.cur_frame().unwrap().precision()
    }

    fn source_pixel_format(&self) -> PixelFormat {
        todo!()
    }
}

impl<C: Channel, const F: u8> JpegDecoder<'_, C, F> {
    fn update_with_marker(&mut self, marker: Marker, data: &mut CursorVec<u8>, data_ranges: Option<Vec<Range<usize>>>) -> Result<(), DecodingError> {
        assert!(data_ranges.is_none() || marker.is_sos(), "Data ranges should only be provided for SOS markers");

        match *marker {
            MarkerType::Sos => {
                let data_ranges = CursorVec::from(data_ranges.expect("Data ranges should be provided for SOS markers"));
                assert_ne!(data_ranges.capacity(), 0, "Data ranges should not be empty for SOS markers");
                let data = std::mem::take(data);

                Scan::update_decoder(self, marker, data, data_ranges)?;
            },
            MarkerType::Sof(_) => {FrameHeader::update_decoder(self, marker, &mut **data)?;},
            MarkerType::Dqt => {QuantizationTables::update_decoder(self, marker, &mut **data)?;},
            MarkerType::Dht => {HuffmanTables::update_decoder(self, marker, &mut **data)?;},
            MarkerType::Dri => {RestartInterval::update_decoder(self, marker, &mut **data)?;},
            _ => (),
        }

        Ok(())
    }

    pub fn push_frame(&mut self, frame: FrameHeader) {
        self.frames.push(frame);
        self.decode_timeline.push(DecodeOp::SetFrame(self.frames.len() - 1));
    }
    pub fn cur_frame(&self) -> Option<&FrameHeader> {self.frames.first()}
    pub fn consume_frame(&mut self) {self.frames.remove(0);}
}

fn check_header<R: Read>(reader: &mut R) -> Result<(), DecodingError> {
    let mut header = [0u8; 2];
    reader.read_exact(&mut header)?;
    if header != JPEG_HEADER {
        return Err(DecodingError::InccorectHeader(header.to_vec()))
    }
    Ok(())
}
