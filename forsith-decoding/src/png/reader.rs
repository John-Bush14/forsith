use std::io::{BufRead, Read};

use crate::{BitBuffer, BufferReader, DecodingError, Num, png::{ChunkData, ChunkType::{self}, checksum::{Adler32, CRC32}, chunks::{IHDR, ZlibHeader, is_chunk_type_critical}}};


const BUFFER_SIZE: usize = 1 << 12;


#[derive(Debug)]
pub struct PngReader<R: BufRead> {
    pub reader: R,
    pub buffer: BufferReader<BUFFER_SIZE>,
    pub crc: CRC32,
    pub adler: Adler32,
    pub(crate) remaining_chunk_bytes: usize,
    cur_type: ChunkType,
    pub bit_buf: BitBuffer<usize>
}

impl<R: BufRead> PngReader<R> {
    pub fn new(reader: R) -> Self {
        let mut reader = Self {
            reader,
            buffer: BufferReader::<BUFFER_SIZE>::new(),
            crc: CRC32::default(),
            adler: Adler32::default(),
            remaining_chunk_bytes: 0,
            cur_type: ChunkType::UnkownAncillerary,
            bit_buf: BitBuffer::<usize>::new()
        };

        let first_len = u32::read_be(&mut reader.reader).unwrap();
        reader.remaining_chunk_bytes = first_len as usize + 4;

        reader.buffer.mut_slice(4).copy_from_slice(&first_len.to_be_bytes());

        reader.fill_buffer::<false>(4).unwrap();

        reader
    }

    pub fn open_chunk(&mut self) -> Result<(), DecodingError> {
        let length = self.buffer.read_be::<u32>();

        let chunk_type_buf = self.buffer.read_array::<4>();
        self.cur_type = match u32::from_be_bytes(chunk_type_buf).try_into() {
            Ok(t) => t,
            Err(_) => {
                if is_chunk_type_critical(&chunk_type_buf) {return Err(DecodingError::UnkownChunk(chunk_type_buf))}
                self.read_exact(&mut vec![0u8; length as usize])?;
                return self.open_chunk();
            }
        };

        Ok(())
    }

    pub fn cur_type(&self) -> ChunkType {self.cur_type}

    pub fn read_chunkdata(&mut self) -> Result<Box<dyn ChunkData>, DecodingError> {
        let chunk_data: Box<dyn ChunkData> = match self.cur_type {
            ChunkType::UnkownAncillerary => unreachable!(),
            ChunkType::Idat => return Ok(Box::new(ZlibHeader::read(self)?)),
            ChunkType::Ihdr => Box::new(IHDR::read(self)?),
            _ => {
                todo!()
            }
        };

        Ok(chunk_data)
    }

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

            if let Ok(t) = ChunkType::try_from(u32::from_be_bytes(type_buf)) {
                if !IDAT && t == ChunkType::Idat {
                    self.buffer.raw_mut_slice(index..index + 4).copy_from_slice(&len_buf);
                    self.buffer.raw_mut_slice(index+4..index + 8).copy_from_slice(&type_buf);

                    return self.fill_buffer::<true>(index + 8)
                }

                if !IDAT && t == ChunkType::Iend {
                    self.buffer.raw_mut_slice(index..index + 4).copy_from_slice(&len_buf);
                    self.buffer.raw_mut_slice(index+4..index + 8).copy_from_slice(&type_buf);

                    let mut crc_buf: [u8; 4] = [0u8; 4];
                    self.reader.read_exact(&mut crc_buf)?;
                    self.validate_crc(u32::from_be_bytes(crc_buf))?;

                    return Ok(())
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


    pub fn read_idat<N: Num>(&mut self) -> Result<N, DecodingError> {
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
    fn peek_bits(&mut self, n: u8) -> std::io::Result<usize> {
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
    fn peek_bits(&mut self, n: u8) -> std::io::Result<usize>;
    fn consume_bits(&mut self, n: u8);
    fn read_bits(&mut self, n: u8) -> std::io::Result<usize> {
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
    type Item = Result<usize, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.reader.read_bits(self.bits))
    }
}
