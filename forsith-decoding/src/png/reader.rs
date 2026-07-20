use std::io::{BufRead, Read};

use crate::{BitBuffer, BufferReader, DecodingError, Int, png::{ChunkType::{self}, checksum::{Adler32, CRC32}, chunks::is_chunk_type_critical}};


const BUFFER_SIZE: usize = 1 << 12;


#[derive(Debug)]
struct Chunk {
    len: usize,
    r#type: ChunkType
}

#[derive(Debug)]
pub struct PngReader<R: BufRead> {
    pub reader: R,
    pub buffer: BufferReader<BUFFER_SIZE>,
    pub crc: CRC32,
    pub adler: Adler32,
    pub(crate) remaining_chunk_bytes: usize,
    cur_chunk: Chunk,
    pub bit_buf: BitBuffer
}

impl<R: BufRead> PngReader<R> {
    pub fn new(reader: R) -> Result<Self, DecodingError> {
        let mut reader = Self {
            reader,
            buffer: BufferReader::<BUFFER_SIZE>::default(),
            crc: CRC32::default(),
            adler: Adler32::default(),
            remaining_chunk_bytes: 0,
            cur_chunk: Chunk {len: 0, r#type: ChunkType::UnkownAncillerary},
            bit_buf: BitBuffer::default()
        };

        let first_len = u32::read_be(&mut reader.reader)?;
        reader.remaining_chunk_bytes = first_len as usize + 4;

        reader.buffer.mut_slice(4).copy_from_slice(&first_len.to_be_bytes());

        reader.fill_buffer::<false>(4)?;

        Ok(reader)
    }

    pub fn open_chunk(&mut self) -> Result<(), DecodingError> {
        self.cur_chunk.len = self.buffer.read_be::<u32>() as usize;

        let chunk_type_buf = self.buffer.read_array::<4>();
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

    fn refill_buffer<const IDAT: bool>(&mut self) -> Result<(), DecodingError> {
        let remaining = self.buffer.remaining();

        self.buffer.empty();

        self.fill_buffer::<IDAT>(remaining)?;

        Ok(())
    }

    fn fill_buffer<const IDAT: bool>(&mut self, mut index: usize) -> Result<(), DecodingError> {
        loop {
            if BUFFER_SIZE - index <= self.remaining_chunk_bytes {
                self.reader.read_exact(self.buffer.raw_mut_slice(index..BUFFER_SIZE))?;

                self.remaining_chunk_bytes -= BUFFER_SIZE - index;

                self.crc.update(self.buffer.raw_slice(index..BUFFER_SIZE));

                break;
            }

            // CRC + next length
            self.reader.read_exact(self.buffer.raw_mut_slice(index..index + self.remaining_chunk_bytes + 4 + 4 + 4))?;
            self.crc.update(self.buffer.raw_slice(index..index + self.remaining_chunk_bytes));

            index += self.remaining_chunk_bytes;

            let crc_buf: [u8; 4] = self.buffer.raw_slice(index..index+4).try_into().unwrap();
            self.validate_crc(u32::from_be_bytes(crc_buf))?;
            self.reset_crc();

            let len_buf: [u8; 4] = self.buffer.raw_slice(index+4..index+8).try_into().unwrap();
            self.remaining_chunk_bytes = u32::from_be_bytes(len_buf) as usize;

            let type_buf: [u8; 4] = self.buffer.raw_slice(index+8..index+12).try_into().unwrap();
            self.update_crc(&type_buf);

            let chunk_type: Result<ChunkType, <ChunkType as TryFrom<u32>>::Error> = ChunkType::try_from(u32::from_be_bytes(type_buf));

            if let Ok(ChunkType::Iend) = chunk_type {
                self.buffer.raw_mut_slice(index..index + 4).copy_from_slice(&len_buf);
                self.buffer.raw_mut_slice(index+4..index + 8).copy_from_slice(&type_buf);

                let mut crc_buf: [u8; 4] = [0u8; 4];
                self.reader.read_exact(&mut crc_buf)?;
                self.validate_crc(u32::from_be_bytes(crc_buf))?;

                return Ok(())
            }

            if let Ok(ChunkType::Idat) = chunk_type {
                if !IDAT {
                    self.buffer.raw_mut_slice(index..index + 4).copy_from_slice(&len_buf);
                    self.buffer.raw_mut_slice(index+4..index + 8).copy_from_slice(&type_buf);

                    return self.fill_buffer::<true>(index + 8)
                }
            } else if IDAT {
                self.buffer.raw_mut_slice(index..index + 4).copy_from_slice(&len_buf);
                self.buffer.raw_mut_slice(index+4..index + 8).copy_from_slice(&type_buf);

                return self.fill_buffer::<false>(index + 8)
            }

            self.buffer.raw_mut_slice(index..index + 4).copy_from_slice(&len_buf);
            self.buffer.raw_mut_slice(index+4..index + 8).copy_from_slice(&type_buf);

            index += 8;
        }

        Ok(())
    }


    pub fn read_idat<N: Int>(&mut self) -> Result<N, DecodingError> {
        if self.buffer.remaining() < std::mem::size_of::<N>() {
            self.refill_buffer::<true>()?;
        }

        Ok(self.buffer.read_le::<N>())
    }
}

impl<R: BufRead> Read for PngReader<R> {
    /// should not be used inside of IDAT chunks
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.buffer.remaining() < buf.len() {
            self.refill_buffer::<false>()?;
        }

        buf.copy_from_slice(self.buffer.slice(buf.len()));
        self.buffer.consume(buf.len());

        Ok(buf.len())
    }
}

impl<R: BufRead> BitReader for PngReader<R> {
    fn peek_bits(&mut self, n: u8) -> std::io::Result<u64> {
        if self.bit_buf.bits_remaining() <= 32 {
            self.fill_bitbuf()?;
        }

        Ok(self.bit_buf.peek(n))
    }

    fn fill_bitbuf(&mut self) -> std::io::Result<()> {
        let refil = self.read_idat::<u32>()?;

        self.bit_buf.push_u32(refil);

        Ok(())
    }
    fn consume_bits(&mut self, n: u8) {
        self.bit_buf.consume(n);
    }
}

pub trait BitReader {
    fn fill_bitbuf(&mut self) -> std::io::Result<()>;
    fn peek_bits(&mut self, n: u8) -> std::io::Result<u64>;
    fn consume_bits(&mut self, n: u8);
    fn read_bits(&mut self, n: u8) -> std::io::Result<u64> {
        let bits = self.peek_bits(n)?;
        self.consume_bits(n);
        Ok(bits)
    }
    fn iterate_bits(&mut self, n: u8) -> BitIterator<'_, Self> where Self: Sized {
        BitIterator {
            reader: self,
            bits: n
        }
    }
}

pub struct BitIterator<'a, R: BitReader> {
    reader: &'a mut R,
    bits: u8
}
impl<R: BitReader> Iterator for BitIterator<'_, R> {
    type Item = Result<u64, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.reader.read_bits(self.bits))
    }
}
