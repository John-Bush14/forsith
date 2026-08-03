use std::io::Read;
use crate::{BufferReader, Channel, DecodingError, PngDecoder, png::{ChunkType::{self}, checksum::CRC32, chunks::ChunkHeader}};

const EXTRA_ALLOC: usize = 1 << 12;

#[derive(Debug)]
pub struct ChunkParser<R: Read> {
    reader: R,
    buffer: BufferReader,
    crc: CRC32,
    cur_chunk: ChunkHeader,
}

impl<R: Read> ChunkParser<R> {
    pub fn new(reader: R) -> Result<Self, DecodingError> {
        Ok(Self {
            reader,
            buffer: BufferReader::new(EXTRA_ALLOC),
            crc: Default::default(),
            cur_chunk: Default::default(),
        })
    }

    pub fn crc(&self) -> CRC32 {self.crc}
    pub fn cur_chunk(&self) -> &ChunkHeader {&self.cur_chunk}

    fn validate_chunkdata(&mut self) -> Result<(), DecodingError> {
        self.crc.update(self.buffer.slice(self.cur_chunk().len()));
        self.buffer.index += self.cur_chunk().len();

        let crc = self.buffer.read_be::<u32>()?;
        self.validate_crc(crc)?;

        Ok(())
    }

    pub fn parse_first_chunk(&mut self) -> Result<(ChunkHeader, &[u8]), DecodingError> {
        self.reader.read_exact(self.buffer.mut_slice(8))?;
        self.parse_chunk_header()?;
        let prev_chunk = self.cur_chunk.clone();

        self.read_chunkdata_and_next_header()?;

        self.validate_chunkdata()?;

        self.parse_chunk_header()?;
        self.buffer.index = 0;

        let chunk_data = self.buffer.raw_slice(8..8 + prev_chunk.len());
        Ok((prev_chunk, chunk_data))
    }

    pub fn parse_chunks<C: Channel, const F: u8>(&mut self, decoder: &mut PngDecoder<'_, C, F>) -> Result<(), DecodingError> {
        let mut reading_idats = matches!(self.cur_chunk().r#type(), ChunkType::Idat);

        while !matches!(self.cur_chunk().r#type(), ChunkType::Iend)  {
            self.read_chunkdata_and_next_header()?;

            self.validate_chunkdata()?;

            if !matches!(self.cur_chunk().r#type(), ChunkType::Idat) {
                self.buffer.index = 0;
                decoder.update_with_chunk(&self.cur_chunk, &mut self.buffer)?;
                self.buffer.index = self.cur_chunk().len() + 4;
            }

            self.parse_chunk_header()?;

            if reading_idats {
                self.buffer.index -= 12;

                if !matches!(self.cur_chunk().r#type(), ChunkType::Idat) {
                    let idat_len = self.buffer.index;

                    self.buffer.index = 0;
                    decoder.update_with_chunk(&ChunkHeader::new(idat_len, ChunkType::Idat), &mut self.buffer)?;
                    self.buffer.index = 0;
                }
            } else {
                self.buffer.index = 0;
            }

           reading_idats = matches!(self.cur_chunk().r#type(), ChunkType::Idat);
        };

        let crc = self.reader.read_be::<u32>()?;
        self.validate_crc(crc)?;

        self.buffer.index = 0;

        Ok(())
    }

    fn read_chunkdata_and_next_header(&mut self) -> Result<(), DecodingError> {
        if self.buffer.remaining() <= self.cur_chunk().len() + 12 {
            self.buffer.expand(self.cur_chunk().len() + 12 + EXTRA_ALLOC);
        }

        // CRC + next (length + type)
        match self.reader.read_exact(self.buffer.raw_mut_slice(self.buffer.index..self.buffer.index + self.cur_chunk().len() + 4 + 4 + 4)) {
            Ok(_) => Ok(()), Err(e) => match e.kind() {
                std::io::ErrorKind::UnexpectedEof => Err(DecodingError::NoIend),
                _ => Err(e.into())
            }
        }
    }

    fn parse_chunk_header(&mut self) -> Result<(), DecodingError> {
        (self.cur_chunk, self.crc) = ChunkHeader::read(&mut self.buffer)?; Ok(())
    }
}
