use std::io::{BufRead, Read};

use crate::{BitBuffer, BufferReader, DecodingError, decompression::BitReader, png::{ChunkType::{self}, checksum::{Adler32, CRC32}, chunks::is_chunk_type_critical}};

const EXTRA_ALLOC: usize = 1 << 12;

#[derive(Debug)]
struct Chunk {
    len: usize,
    r#type: ChunkType
}

#[derive(Debug)]
pub struct PngReader<R: Read> {
    pub reader: R,
    pub buffer: BufferReader,
    pub crc: CRC32,
    pub adler: Adler32,
    pub(crate) remaining_chunk_bytes: usize,
    cur_chunk: Chunk,
    pub bit_buf: BitBuffer
}

impl<R: Read> PngReader<R> {
    pub fn new(reader: R) -> Result<Self, DecodingError> {
        let mut reader = Self {
            reader,
            buffer: BufferReader::new(EXTRA_ALLOC),
            crc: CRC32::default(),
            adler: Adler32::default(),
            remaining_chunk_bytes: 0,
            cur_chunk: Chunk {len: 0, r#type: ChunkType::UnkownAncillerary},
            bit_buf: BitBuffer::default()
        };

        reader.read_into_buffer()?;

        Ok(reader)
    }

    pub fn open_chunk(&mut self) -> Result<(), DecodingError> {
        self.cur_chunk.len = self.buffer.read_be::<u32>()? as usize;

        let chunk_type_buf = self.buffer.read_array::<4>()?;
        self.cur_chunk.r#type = match u32::from_be_bytes(chunk_type_buf).try_into() {
            Ok(t) => t,
            Err(_) => {
                if is_chunk_type_critical(&chunk_type_buf) {return Err(DecodingError::UnkownChunk(chunk_type_buf))}

                self.read_exact(&mut vec![0u8; self.cur_chunk.len])?;
                return self.open_chunk();
            }
        };

        Ok(())
    }

    pub fn cur_chunk_type(&self) -> ChunkType {self.cur_chunk.r#type}
    pub fn cur_chunk_len(&self) -> usize {self.cur_chunk.len}

    fn prepare_buffer(&mut self) -> Result<(), DecodingError> {
        let first_len = self.reader.read_be::<u32>()?;
        self.remaining_chunk_bytes = first_len as usize + 4;

        self.buffer.mut_slice(4).copy_from_slice(&first_len.to_be_bytes());
        self.buffer.index += 4;

        Ok(())
    }

    fn validate_chunkdata(&mut self) -> Result<(), DecodingError> {
        self.crc.update(self.buffer.slice(self.remaining_chunk_bytes));
        self.buffer.index += self.remaining_chunk_bytes;

        let crc = self.buffer.read_be::<u32>()?;
        self.validate_crc(crc)?;
        self.reset_crc();

        Ok(())
    }

    pub fn read_into_buffer(&mut self) -> Result<(), DecodingError> {
        self.prepare_buffer()?;

        let mut reading_idats = false;
        #[allow(clippy::useless_conversion)]
        let mut chunk_type: Result<ChunkType, <ChunkType as TryFrom<u32>>::Error> = Ok(ChunkType::UnkownAncillerary).into();

        while !matches!(chunk_type, Ok(ChunkType::Iend))  {
            self.read_chunkdata_and_next_header()?;

            self.validate_chunkdata()?;

            self.remaining_chunk_bytes = self.buffer.read_be::<u32>()? as usize;

            let type_buf: [u8; 4] = self.buffer.read_array()?;
            self.update_crc(&type_buf);
            chunk_type = ChunkType::try_from(u32::from_be_bytes(type_buf));

            match chunk_type {
                Ok(ChunkType::Idat) if reading_idats => self.buffer.index -= 12,
                _ => self.place_chunk_header(type_buf),
            }

           reading_idats = matches!(chunk_type, Ok(ChunkType::Idat))
        };

        let crc = self.reader.read_be::<u32>()?;
        self.validate_crc(crc)?;

        self.buffer.index = 0;

        Ok(())
    }

    fn place_chunk_header(&mut self, type_buf: [u8; 4]) {
        self.buffer.index -= 4;

        let index = self.buffer.index - 8;
        self.buffer.raw_mut_slice(index..index + 4).copy_from_slice(&(self.remaining_chunk_bytes as u32).to_be_bytes());
        self.buffer.raw_mut_slice(index+4..index + 8).copy_from_slice(&type_buf);
    }

    pub fn align(&mut self) -> Result<(), DecodingError> {
        let alignment = 4 - (self.buffer.index % align_of::<u32>());

        let mut buf = vec![0u8; alignment];
        self.read_exact(&mut buf)?;

        for b in buf {self.bit_buf.push(b)}; Ok(())
    }

    fn read_chunkdata_and_next_header(&mut self) -> Result<(), DecodingError> {
        if self.buffer.remaining() <= self.remaining_chunk_bytes + 12 {
            self.buffer.expand(self.remaining_chunk_bytes + 12 + EXTRA_ALLOC);
        }

        // CRC + next (length + type)
        match self.reader.read_exact(self.buffer.raw_mut_slice(self.buffer.index..self.buffer.index + self.remaining_chunk_bytes + 4 + 4 + 4)) {
            Ok(_) => Ok(()), Err(e) => match e.kind() {
                std::io::ErrorKind::UnexpectedEof => Err(DecodingError::NoIend),
                _ => Err(e.into())
            }
        }
    }

    pub fn unconsume_bitbuf(&mut self) {
        self.consume_bits(self.bit_buf.bits_remaining() % 8);

        let bitbuf_bytes = self.bit_buf.bits_remaining() as usize / 8;

        self.buffer.unconsume(bitbuf_bytes);
        self.bit_buf.consume(bitbuf_bytes as u8);
    }
}

impl<R: Read> Read for PngReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        buf.copy_from_slice(self.buffer.slice(buf.len()));
        self.buffer.consume(buf.len());

        Ok(buf.len())
    }
}

impl<R: Read> BitReader for PngReader<R> {
    #[inline(always)]
    fn peek_bits(&mut self, n: u8) -> u64 {
        if self.bit_buf.bits_remaining() <= 32 {
            self.fill_bitbuf();
        }

        self.bit_buf.peek(n)
    }

    #[inline(always)]
    fn fill_bitbuf(&mut self) {
        let refil = u32::from_le_bytes(self.buffer.buffer[self.buffer.index..self.buffer.index + 4].try_into().unwrap());
        self.buffer.consume(4);

        self.bit_buf.push(refil);
    }

    #[inline(always)]
    fn consume_bits(&mut self, n: u8) {
        self.bit_buf.consume(n);
    }

    #[inline]
    fn remaining_bits(&self) -> u8 {
        self.bit_buf.bits_remaining
    }

    #[inline(always)]
    fn peek_bits_nobranch(&mut self, n: u8) -> u64 {self.bit_buf.peek(n)}
}
