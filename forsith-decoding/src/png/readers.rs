use std::{cmp::min, io::{BufRead, Read}, usize};

use crate::{DecodingError, Num, png::{ChunkData, ChunkType, checksum::Adler32, chunks::{IHDR, ZlibHeader, is_chunk_type_critical}}, read_exact_array};

#[derive(Debug)]
pub struct ChunkReader<R: BufRead> {
    pub reader: R,
    pub crc: u32,
    pub adler: Adler32,
    pub reading_data: bool,
    remaining_bytes: u32,
    cur_type: ChunkType,
    bit_buf: BitBuffer
}

impl<R: BufRead> ChunkReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            crc: 0,
            adler: Adler32::new(),
            reading_data: false,
            remaining_bytes: 0,
            cur_type: ChunkType::UnkownAncillerary,
            bit_buf: BitBuffer::new()
        }
    }
    pub fn normal_reader(&mut self) -> &mut R {&mut self.reader}

    pub fn close_chunk(&mut self) -> Result<(), DecodingError> {
        if self.remaining_bytes != 0 {
            return Err(DecodingError::EarlyClose(self.cur_type, self.remaining_bytes));
        }

        self.validate_crc()
    }

    pub fn open_chunk(&mut self) -> Result<(), DecodingError> {
        let length = u32::read_be(self.normal_reader())?;
        self.remaining_bytes = 4 + length; // for type field.

        self.init_crc();

        let chunk_type_buf = read_exact_array::<4, _>(self)?;
        self.cur_type = match u32::from_be_bytes(chunk_type_buf).try_into() {
            Ok(t) => t,
            Err(_) => {
                self.read_exact(&mut vec![0u8; length as usize])?;
                self.close_chunk()?;
                if is_chunk_type_critical(&chunk_type_buf) {return Err(DecodingError::UnkownChunk(chunk_type_buf))}
                ChunkType::UnkownAncillerary
            }
        }; Ok(())
    }

    pub fn cur_type(&self) -> ChunkType {self.cur_type}

    pub fn read_data(&mut self) -> Result<Box<dyn ChunkData>, DecodingError> {
        let chunk_data: Box<dyn ChunkData> = match self.cur_type {
            ChunkType::UnkownAncillerary => unreachable!(),
            ChunkType::Idat => return Ok(Box::new(ZlibHeader::read(self)?)),
            ChunkType::Ihdr => Box::new(IHDR::read(self)?),
            _ => {
                todo!()
            }
        };

        self.close_chunk()?;

        Ok(chunk_data)
    }

    fn skip_idat_boundrary(&mut self) -> std::io::Result<()> {
        self.close_chunk()?;
        self.open_chunk()?;

        if self.cur_type != ChunkType::Idat {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Zlib stream ended undexpectedly"));
        }

        Ok(())
    }
}

impl<R: BufRead> Read for ChunkReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut len = min(buf.len(), self.remaining_bytes as usize);

        self.remaining_bytes -= len as u32;
        self.reader.read_exact(&mut buf[..len])?;

        self.update_crc(&buf[..len]);

        if len != buf.len() && self.cur_type == ChunkType::Idat {
            self.skip_idat_boundrary()?;

            len = self.read(&mut buf[len..])? + len;
        }

        if self.reading_data {
            self.update_adler32(&buf[..len]);
        }

        Ok(len)
    }
}

impl<R: BufRead> BufRead for ChunkReader<R> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if self.remaining_bytes == 0 && self.cur_type == ChunkType::Idat {
            self.skip_idat_boundrary()?;
        }

        let buf = self.reader.fill_buf()?;
        let len = min(buf.len(), self.remaining_bytes as usize);

        Ok(&buf[..len])
    }

    fn consume(&mut self, amt: usize) {
        self.remaining_bytes -= amt as u32;
        self.reader.consume(amt);
    }
}

impl<R: BufRead> BitReader for ChunkReader<R> {
    fn peek_bits(&mut self, n: u8) -> std::io::Result<usize> {
        if self.bit_buf.bits_remaining <= n {
            self.fill_bitbuf()?;
        }

        Ok(self.bit_buf.peek(n))
    }

    fn fill_bitbuf(&mut self) -> std::io::Result<()> {self.bit_buf.fill(&mut self.reader)}
    fn consume_bits(&mut self, n: u8) {self.bit_buf.consume(n);}
}

pub trait BitReader: BufRead {
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
        return Some(self.reader.read_bits(self.bits))
    }
}

#[derive(Debug)]
struct BitBuffer {
    buf: usize,
    bits_remaining: u8
}
impl BitBuffer {
    fn new() -> Self {
        Self {
            buf: 0,
            bits_remaining: 0
        }
    }

    fn fill<R: BufRead>(&mut self, reader: &mut R) -> std::io::Result<()> {
        let buf = reader.fill_buf()?;
        if buf.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Unexpected EOF while reading bits"));
        }

        let len = min(buf.len(), std::mem::size_of::<usize>() - (self.bits_remaining/8) as usize);

        for b in &buf[..len] {self.push(*b);}

        Ok(())
    }

    fn peek(&self, n: u8) -> usize {
        self.buf & ((1 << n) - 1)
    }

    fn consume(&mut self, n: u8) {
        self.buf >>= n as usize;
        self.bits_remaining -= n;
    }

    fn push(&mut self, byte: u8) {
        self.buf |= (byte as usize) << (self.bits_remaining as usize);
        self.bits_remaining += 8;
    }
}
