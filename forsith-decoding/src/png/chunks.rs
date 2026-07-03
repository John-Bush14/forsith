use std::{any::Any, fmt::Display, io::Read};
use crate::{DecodingError, Num, PngDecoder, png::{ChunkType::UnkownAncillerary, ColorType, crc::CRCReader}, read_exact_array};
use num_enum::{TryFromPrimitive, IntoPrimitive};

#[repr(u32)]
#[derive(TryFromPrimitive, IntoPrimitive, Clone, Copy, Debug, PartialEq)]
pub enum ChunkType {
    Ihdr = 0x49484452,
    Plte = 0x504C5445,
    Idat = 0x49444154,
    Iend = 0x49454E44,
    UnkownAncillerary
}
impl Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(u32::from(*self).to_be_bytes().iter().map(|b| *b as char).collect::<String>().as_str())
    }
}
impl ChunkType {
    pub fn is_critical(&self) -> bool {
        *self as u32 & 0x20000000 == 0
    }
}
pub fn is_chunk_type_critical(chunk_type_buffer: &[u8; 4]) -> bool {
    chunk_type_buffer[0] & 0x20 == 0
}

#[derive(Debug)]
pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
}
impl Chunk {
    pub fn open<R: Read>(reader: &mut CRCReader<R>) -> Result<Self, DecodingError> {
        let length = u32::read_be(reader.normal_reader())?;

        let chunk_type_buf = read_exact_array::<4, _>(reader)?;
        let chunk_type: ChunkType = match u32::from_be_bytes(chunk_type_buf).try_into() {
            Ok(t) => t,
            Err(_) => {
                reader.read_exact(&mut vec![0u8; length as usize])?;
                reader.validate_crc()?;
                if is_chunk_type_critical(&chunk_type_buf) {return Err(DecodingError::UnkownChunk(chunk_type_buf))}
                ChunkType::UnkownAncillerary
            }
        };

        Ok(Self {
            length,
            chunk_type,
        })
    }

    pub fn r#type(&self) -> ChunkType {self.chunk_type}

    pub fn read_data<R: Read>(&mut self, reader: &mut CRCReader<R>) -> Result<Box<dyn ChunkData>, DecodingError> {
        let chunk_data: Box<dyn ChunkData> = match self.chunk_type {
            ChunkType::UnkownAncillerary => unreachable!(),
            ChunkType::Ihdr => Box::new(IHDR::read(reader)?),
            _ => {
                todo!()
            }
        };

        reader.validate_crc()?;

        Ok(chunk_data)
    }
}



pub trait ChunkData: Any {
    fn chunk_type(&self) -> ChunkType; // &self needed for Box

    fn validate(&self) -> Result<(), DecodingError>;

    fn read<R: Read>(data: &mut R) -> Result<Self, DecodingError>
    where Self: Sized;
}
pub fn downcast_chunkdata<T: ChunkData + Any>(b: Box<dyn ChunkData>) -> Result<Box<T>, Box<dyn Any>> {
    let b: Box<dyn Any> = b;
    b.downcast::<T>()
}

#[derive(Debug)]
pub struct IHDR {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: ColorType,
    compression_method: u8,
    filter_method: u8,
    interlace_method: u8,
}

impl ChunkData for IHDR {
    fn chunk_type(&self) -> ChunkType {ChunkType::Ihdr}

    fn validate(&self) -> Result<(), DecodingError> {
        if !matches!((self.compression_method, self.filter_method, self.interlace_method), (0, 0, 0 | 1)) {
            return Err(DecodingError::InvalidChunk(self.chunk_type()));
        }

        if self.interlace_method == 1 {
            return Err(DecodingError::InteralacingNotSupported);
        }

        match (&self.bit_depth, &self.color_type) {
            (1 | 2 | 4 | 8 | 16, ColorType::Grayscale)
            | (8 | 16, ColorType::Truecolor)
            | (1 | 2 | 4 | 8, ColorType::Indexed)
            | (8 | 16, ColorType::GrayscaleAlpha)
            | (8 | 16, ColorType::TruecolorAlpha) => Ok(()),
            _ => Err(DecodingError::InvalidChunk(self.chunk_type())),
        }
    }

    fn read<R: Read>(data: &mut R) -> Result<Self, DecodingError>
    where Self: Sized {
        Ok(Self {
            width: u32::read_be(data)?,
            height: u32::read_be(data)?,
            bit_depth: u8::read_be(data)?,
            color_type: ColorType::try_from(u8::read_be(data)?).map_err(|_| DecodingError::InvalidChunk(ChunkType::Ihdr))?,
            compression_method: u8::read_be(data)?,
            filter_method:  u8::read_be(data)?,
            interlace_method: u8::read_be(data)?,
        })
    }
}
