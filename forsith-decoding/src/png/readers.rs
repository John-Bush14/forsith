use std::{cmp::min, io::{BufRead, Read}};

use crate::{BitBuffer, DecodingError, Num, png::{ChunkData, ChunkType, checksum::{Adler32, CRC32}, chunks::{IHDR, ZlibHeader, is_chunk_type_critical}}, read_exact_array};

#[derive(Debug)]
pub struct ChunkReader<R: BufRead> {
    pub reader: R,
    pub crc: CRC32,
    pub adler: Adler32,
    pub(crate) remaining_bytes: u32,
    cur_type: ChunkType,
    pub bit_buf: BitBuffer<usize>
}

impl<R: BufRead> ChunkReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            crc: CRC32::default(),
            adler: Adler32::default(),
            remaining_bytes: 0,
            cur_type: ChunkType::UnkownAncillerary,
            bit_buf: BitBuffer::<usize>::new()
        }
    }
    pub fn normal_reader(&mut self) -> &mut R {&mut self.reader}

    pub fn close_chunk(&mut self) -> Result<(), DecodingError> {
        if self.remaining_bytes != 0 {
            return Err(DecodingError::IncorrectClose(self.cur_type, self.remaining_bytes));
        }

        self.validate_crc()
    }

    pub fn open_chunk(&mut self) -> Result<(), DecodingError> {
        let length = u32::read_be(self.normal_reader())?;
        self.remaining_bytes = 4 + length; // for type field.

        self.reset_crc();

        let chunk_type_buf = read_exact_array::<4, _>(self)?;
        self.cur_type = match u32::from_be_bytes(chunk_type_buf).try_into() {
            Ok(t) => t,
            Err(_) => {
                self.read_exact(&mut vec![0u8; length as usize])?;
                self.close_chunk()?;
                if is_chunk_type_critical(&chunk_type_buf) {return Err(DecodingError::UnkownChunk(chunk_type_buf))}
                return self.open_chunk();
            }
        };

        Ok(())
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
        let consumed_data = &self.reader.fill_buf().unwrap()[..amt];
        self.crc.update(consumed_data);
        self.remaining_bytes -= amt as u32;
        self.reader.consume(amt);
    }
}

impl<R: BufRead> BitReader for ChunkReader<R> {
    fn peek_bits(&mut self, n: u8) -> std::io::Result<usize> {
        if self.bit_buf.bits_remaining <= n {
            self.fill_bitbuf(n)?;
        }

        Ok(self.bit_buf.peek(n))
    }

    fn fill_bitbuf(&mut self, n: u8) -> std::io::Result<()> {
        let needed_bytes = (n - self.bit_buf.bits_remaining) / 8 + if !(n - self.bit_buf.bits_remaining).is_multiple_of(8) {1} else {0};

        for _ in 0..needed_bytes {
            let byte = self.fill_buf()?[0];
            self.bit_buf.push(byte);
            self.consume(1);
        }

        Ok(())
    }
    fn consume_bits(&mut self, n: u8) {self.bit_buf.consume(n);}
}

pub trait BitReader: BufRead {
    fn fill_bitbuf(&mut self, n: u8) -> std::io::Result<()>;
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
