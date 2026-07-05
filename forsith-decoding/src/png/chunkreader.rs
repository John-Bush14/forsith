use std::{cmp::min, io::Read};

use crate::{DecodingError, png::{ChunkData, ChunkType, chunks::{IHDR, is_chunk_type_critical}}, read_exact_array, Num};

#[derive(Debug)]
pub struct ChunkReader<R: Read> {
    pub reader: R,
    pub crc: u32,
    alder32: u32,
    remaining_bytes: u32,
    cur_type: ChunkType
}

impl<R: Read> ChunkReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            crc: 0,
            alder32: 0,
            remaining_bytes: 0,
            cur_type: ChunkType::UnkownAncillerary
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
            ChunkType::Ihdr => Box::new(IHDR::read(self)?),
            _ => {
                todo!()
            }
        };

        self.close_chunk()?;

        Ok(chunk_data)
    }
}

impl<R: Read> Read for ChunkReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = min(buf.len(), self.remaining_bytes as usize);

        self.remaining_bytes -= len as u32;
        self.reader.read_exact(&mut buf[..len])?;

        self.update_crc(&buf[..len]);

        if len != buf.len() && self.cur_type == ChunkType::Idat {
            self.close_chunk()?;
            self.open_chunk()?;

            return self.read(&mut buf[len..]).map(|n| n + len);
        }

        Ok(len)
    }
}
