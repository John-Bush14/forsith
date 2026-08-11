use std::io::{Read, Seek};
use derive_more::Deref;

use crate::{DecodingError, buffers::CursorVec, parsing::{SegmentHeader, SegmentParser}, png::{ChunkType::{self}, checksum::CRC32}};

const BASE_ALLOC: usize = 1 << 12;

#[derive(Debug)]
pub struct ChunkParser<R: Read> {
    reader: R,
    buffer: CursorVec<u8>,
    header: ChunkHeader,
}

impl<R: Read> ChunkParser<R> {
    pub fn new(reader: R) -> Result<Self, DecodingError> {
        Ok(Self {
            reader,
            buffer: CursorVec::<u8>::new(BASE_ALLOC),
            header: Default::default(),
        })
    }

    fn validate_crc(&mut self, stored_crc: u32) -> Result<(), DecodingError> {
        self.header.crc.validate(stored_crc)
    }

    fn coalesce_idat_chunks(&mut self) -> Result<(), DecodingError> {
        while self.header.is_idat() {
            self.read_to_next_header()?;
            self.parse_header()?;
            self.buffer.seek_relative(-12).unwrap();
        } Ok(())
    }

    fn handle_idat_chunks<F>(&mut self, out: &mut F) -> Result<(), DecodingError>
        where F: FnMut(&ChunkHeader, &mut CursorVec<u8>) -> Result<(), DecodingError>
    {
        let idat_start = self.buffer.cursor();

        self.coalesce_idat_chunks()?;

        let idat_len = self.buffer.cursor() - idat_start;

        self.buffer.seek_relative(-(idat_len as i64)).unwrap();
        out(&ChunkHeader::new(idat_len, ChunkType::Idat), &mut self.buffer)
    }
}

impl<R: Read> SegmentParser<R> for ChunkParser<R> {
    type Header = ChunkHeader;

    fn context<'s, 'a, 'b, 'c>(&'s mut self) -> (&'a mut CursorVec<u8>, &'b mut R, &'c mut Self::Header)
        where 's: 'a, 's: 'b, 's: 'c
    {
        (&mut self.buffer, &mut self.reader, &mut self.header)
    }

    fn validate_segment(&mut self) -> Result<(), DecodingError> {
        self.header.crc.update(self.buffer.take_slice(self.header.len()));

        let crc = self.buffer.read_be::<u32>()?;
        self.validate_crc(crc)?;

        Ok(())
    }

    /// out should ensure whole chunk is read before returing, for cursor alignment.
    fn parse_chunks<F>(&mut self, mut out: F) -> Result<(), DecodingError>
        where F: FnMut(&Self::Header, &mut CursorVec<u8>) -> Result<(), DecodingError>
    {
        if self.header.is_idat() {self.handle_idat_chunks(&mut out)?}

        while !self.header.is_iend() {
            self.read_to_next_header()?;

            self.buffer.seek_relative(-(self.header.length() as i64)).unwrap();
            out(&self.header, &mut self.buffer)?;
            self.buffer.seek_relative(4)?;

            self.parse_header()?;

            if self.header.is_idat() {self.handle_idat_chunks(&mut out)?}

            self.buffer.set_cursor(0);
        };

        let crc = self.reader.read_be::<u32>()?;
        self.validate_crc(crc)?;

        Ok(())
    }
}

#[derive(Default, Debug, Clone, Deref)]
pub struct ChunkHeader {
    len: usize,
    #[deref]
    r#type: ChunkType,
    pub crc: CRC32,
}
impl ChunkHeader {
    pub fn new(len: usize, r#type: ChunkType) -> Self {Self {len, r#type, crc: CRC32::default()}}

    pub fn len(&self) -> usize {self.len}

    pub fn r#type(&self) -> ChunkType {self.r#type}
}
impl SegmentHeader for ChunkHeader {
    const SIZE: usize = 8;

    fn length(&self) -> usize {self.len + 4}

    fn read<R: Read>(reader: &mut R) -> Result<Self, DecodingError> {
        let len = reader.read_be::<u32>()? as usize;

        let chunk_type_buf = reader.read_array::<4>()?;
        let mut crc = CRC32::default(); crc.update(&chunk_type_buf);

        let r#type = match u32::from_be_bytes(chunk_type_buf).try_into() {
            Ok(t) => t,
            Err(_) => {
                if is_chunk_type_critical(&chunk_type_buf) {return Err(DecodingError::UnkownChunk(chunk_type_buf))}

                ChunkType::UnkownAncillerary
            }
        };

        Ok(Self {len, r#type, crc})
    }
}

pub fn is_chunk_type_critical(chunk_type_buffer: &[u8; 4]) -> bool {
    chunk_type_buffer[0] & 0x20 == 0
}
