use std::io::{Read, Seek};
use crate::{Channel, DecodingError, PngDecoder, buffers::CursorVec, png::{ChunkType::{self}, checksum::CRC32, chunks::ChunkHeader}};

const EXTRA_ALLOC: usize = 1 << 12;

#[derive(Debug)]
pub struct ChunkParser<R: Read> {
    reader: R,
    buffer: CursorVec<u8>,
    crc: CRC32,
    cur_chunk: ChunkHeader,
}

impl<R: Read> ChunkParser<R> {
    pub fn new(reader: R) -> Result<Self, DecodingError> {
        Ok(Self {
            reader,
            buffer: CursorVec::<u8>::new(EXTRA_ALLOC),
            crc: Default::default(),
            cur_chunk: Default::default(),
        })
    }

    pub fn crc(&self) -> CRC32 {self.crc}
    pub fn cur_chunk(&self) -> &ChunkHeader {&self.cur_chunk}

    fn validate_chunkdata(&mut self) -> Result<(), DecodingError> {
        self.crc.update(self.buffer.take_slice(self.cur_chunk().len()));

        let crc = self.buffer.read_be::<u32>()?;
        self.validate_crc(crc)?;

        Ok(())
    }

    pub fn parse_first_chunk(&mut self) -> Result<(ChunkHeader, &[u8]), DecodingError> {
        self.buffer.fill_from(&mut self.reader, 8)?;
        self.parse_chunk_header()?;
        let prev_chunk = self.cur_chunk.clone();

        self.read_chunkdata_and_next_header()?;

        self.validate_chunkdata()?;

        self.parse_chunk_header()?;
        self.buffer.set_cursor(0);

        let chunk_data = &self.buffer.get_ref()[8..8 + prev_chunk.len()];
        Ok((prev_chunk, chunk_data))
    }

    pub fn parse_chunks<C: Channel, const F: u8>(&mut self, decoder: &mut PngDecoder<'_, C, F>) -> Result<(), DecodingError> {
        let mut reading_idats = self.cur_chunk().is_idat();

        while !self.cur_chunk().is_iend() {
            self.read_chunkdata_and_next_header()?;

            self.validate_chunkdata()?;

            if !self.cur_chunk().is_idat() {
                self.buffer.set_cursor(0);
                decoder.update_with_chunk(&self.cur_chunk, &mut self.buffer)?;
                self.buffer.set_cursor(self.cur_chunk().len() + 4);
            }

            self.parse_chunk_header()?;

            if reading_idats {
                self.buffer.seek_relative(-12).unwrap();

                if !self.cur_chunk().is_idat() {
                    let idat_len = self.buffer.cursor();

                    self.buffer.set_cursor(0);
                    decoder.update_with_chunk(&ChunkHeader::new(idat_len, ChunkType::Idat), &mut self.buffer)?;
                    self.buffer.set_cursor(0);
                }
            } else {
                self.buffer.set_cursor(0);
            }

           reading_idats = self.cur_chunk().is_idat();
        };

        let crc = self.reader.read_be::<u32>()?;
        self.validate_crc(crc)?;

        self.buffer.set_cursor(0);

        Ok(())
    }

    fn read_chunkdata_and_next_header(&mut self) -> Result<(), DecodingError> {
        if self.buffer.remaining() <= self.cur_chunk().len() + 12 {
            self.buffer.expand(self.cur_chunk().len() + 12 + EXTRA_ALLOC);
        }

        // CRC + next (length + type)
        match self.buffer.fill_from(&mut self.reader, self.cur_chunk.len() + 4 + 8) {
            Ok(_) => Ok(()), Err(e) => match e.kind() {
                std::io::ErrorKind::UnexpectedEof => Err(DecodingError::NoIend),
                _ => Err(e.into())
            }
        }
    }

    fn parse_chunk_header(&mut self) -> Result<(), DecodingError> {
        (self.cur_chunk, self.crc) = ChunkHeader::read(&mut *self.buffer)?; Ok(())
    }
}
