use std::{array, io::Read, ops::Range};
use crate::{Channel, DecodingError, ImageDecoder, PixelFormat, buffers::{CursorVec, OutputWriter}, jpeg::{idct::IdctTable, markers::{FrameHeader, HuffmanTables, MarkerType, QuantizationTables, RestartInterval, Scan}, parser::Marker}, parsing::{SegmentHeader, SegmentParser}};

pub mod markers;

mod idct;

mod parser;
use derive_more::IsVariant;
use num_enum::{FromPrimitive, TryFromPrimitive};
use parser::JpegParser;

type HuffmanTree = crate::decompression::HuffmanTree<16, 10,  16_384>;

const JPEG_HEADER: [u8; 2] = [0xFF, 0xD8];

#[derive(Debug)]
pub struct JpegDecoder<'a, C: Channel, const F: u8> {
    phantom: std::marker::PhantomData<&'a C>,
    frames: CursorVec<FrameHeader>,
    decode_timeline: Vec<DecodeOp>,
    huffman_trees: [[Box<HuffmanTree>; 4]; 2],
    idct_tables: [IdctTable; 4],
    restart_interval: Option<u16>,
}

#[repr(u8)]
#[derive(Debug, TryFromPrimitive, IsVariant)]
enum HuffmanTreeType {
    Dc = 0,
    Ac = 1
}

#[derive(Debug, IsVariant)]
enum DecodeOp {
    SetFrame(usize),
    SetQuantizationTable(usize, Box<IdctTable>),
    SetHuffmanTree(HuffmanTreeType, usize, Box<[u8; 256]>),
    Scan(Box<Scan>),
    SetRestartInterval(u16),
}

impl<'a, C: Channel, const F: u8> ImageDecoder<'a, C, F> for JpegDecoder<'a, C, F> {
    fn open_validated<R: Read>(mut reader: R) -> Result<Self, DecodingError> where Self: Sized {
        check_header(&mut reader)?;

        let mut parser = JpegParser::new(reader);

        let _ = parser.parse_first_chunk()?;

        let mut decoder = Self {
            phantom: std::marker::PhantomData,
            frames: CursorVec::default(),
            decode_timeline: Vec::new(),
            huffman_trees: array::from_fn(|_| array::from_fn(|_| Box::new(HuffmanTree::default()))),
            idct_tables: [IdctTable::DEFAULT; 4],
            restart_interval: None,
        };

        parser.parse_chunks(|header, data, data_ranges| decoder.update_with_marker(*header, data, data_ranges))?;

        if decoder.frames.is_empty() {
            return Err(DecodingError::NoFrame);
        }

        Ok(decoder)
    }

    fn read(&mut self, buf: &mut [<C as Channel>::StorageType]) -> Result<usize, DecodingError> {
        let output = OutputWriter::<'_, C, F>::new(buf);

        while let Some(op) = self.get_decodeop() {
            if op.is_scan() {
                todo!();
            }

            let op = self.pop_decodeop();

            match op {
                DecodeOp::SetFrame(frame_index) => {
                    self.frames.set_cursor(frame_index);
                },
                DecodeOp::SetQuantizationTable(table_index, idct_table) => {
                    self.idct_tables[table_index] = *idct_table;
                },
                DecodeOp::SetHuffmanTree(tree_type, table_index, tree_data) => {
                    self.huffman_trees[tree_type as u8 as usize][table_index].load(&*tree_data)?;
                },
                DecodeOp::SetRestartInterval(interval) => {
                    self.restart_interval = Some(interval);
                }
                DecodeOp::Scan(_) => unreachable!()
            }
            self.pop_decodeop();
        }

        Ok(output.len())
    }

    fn image_dimensions(&self) -> (usize, usize) {
        let dim = self.cur_frame().unwrap().dimensions();
        (dim.0 as usize, dim.1 as usize)
    }

    fn min_buf_size(&self) -> usize {
        let warning = 0;
        self.max_buf_size()
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
                assert!(!data_ranges.is_empty(), "Data ranges should not be empty for SOS markers");
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

    fn push_frame(&mut self, frame: FrameHeader) {
        self.frames.get_mut().push(frame);
        self.decode_timeline.push(DecodeOp::SetFrame(self.frames.capacity() - 1));
    }
    #[must_use]
    fn cur_frame(&self) -> Option<&FrameHeader> {self.frames.current()}

    fn push_decodeop(&mut self, op: DecodeOp) {self.decode_timeline.push(op);}
    fn get_decodeop(&self) -> Option<&DecodeOp> {self.decode_timeline.first()}
    fn pop_decodeop(&mut self) -> DecodeOp {self.decode_timeline.remove(0)}
}

fn check_header<R: Read>(reader: &mut R) -> Result<(), DecodingError> {
    let mut header = [0u8; 2];
    reader.read_exact(&mut header)?;
    if header != JPEG_HEADER {
        return Err(DecodingError::InccorectHeader(header.to_vec()))
    }
    Ok(())
}
