use std::io::{Read, Seek};
use derive_more::Deref;

use crate::{DecodingError, buffers::CursorVec, image::png::{ChunkType::{self}, checksums::CRC32}, parsing::{SegmentHeader, SegmentParser}};

const BASE_ALLOC: usize = 1 << 12;

#[derive(Debug)]
pub struct ChunkParser<R: Read> {
    reader: R,
    buffer: CursorVec<u8>,
    header: ChunkHeader,
}

impl<R: Read> ChunkParser<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: CursorVec::<u8>::new(BASE_ALLOC),
            header: ChunkHeader::default(),
        }
    }

    fn validate_crc(&self, stored_crc: u32) -> Result<(), DecodingError> {
        self.header.crc.validate(stored_crc)
    }

    fn coalesce_idat_chunks(&mut self) -> Result<(), DecodingError> {
        while self.header.is_idat() {
            self.read_to_next_header()?;
            self.parse_header()?;
            self.buffer.seek_relative(-12).unwrap();
        } Ok(())
    }
}

impl<R: Read> SegmentParser<R> for ChunkParser<R> {
    type Header = ChunkHeader;
    type ExtraOut = ();

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

    fn handle_special_segment<F>(&mut self, out: &mut F) -> Result<(), DecodingError>
        where F: FnMut(&ChunkHeader, &mut CursorVec<u8>, ()) -> Result<(), DecodingError>
    {
        let idat_start = self.buffer.cursor();

        self.coalesce_idat_chunks()?;

        let idat_len = self.buffer.cursor() - idat_start;

        self.buffer.unconsume(idat_len);
        out(&ChunkHeader::new(idat_len, ChunkType::Idat), &mut self.buffer, ())
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

    pub const fn len(&self) -> usize {self.len}

    pub const fn r#type(&self) -> ChunkType {self.r#type}
}
impl SegmentHeader for ChunkHeader {
    const MAX_SIZE: usize = 8;

    fn length(&self) -> usize {self.len + 4}

    fn read<R: Read>(reader: &mut R) -> Result<Self, DecodingError> {
        let len = reader.read_be::<u32>()? as usize;

        let chunk_type_buf = reader.read_array::<4>()?;
        let mut crc = CRC32::default(); crc.update(&chunk_type_buf);

        let r#type = if let Ok(t) = u32::from_be_bytes(chunk_type_buf).try_into() {
            t
        } else {
            if is_chunk_type_critical(chunk_type_buf) {return Err(DecodingError::UnkownChunk(chunk_type_buf))}

            ChunkType::UnkownAncillerary
        };

        Ok(Self {len, r#type, crc})
    }

    fn is_final(&self) -> bool {self.is_iend()}
    fn is_special(&self) -> bool {self.is_idat()}
}

pub const fn is_chunk_type_critical(chunk_type_buffer: [u8; 4]) -> bool {
    chunk_type_buffer[0] & 0x20 == 0
}
