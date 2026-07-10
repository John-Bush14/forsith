use std::io::{BufRead, Read};
use crate::{DecodingError, HistoryBuffer, ImageDecoder, Num, PixelFormat, png::{chunks::{IHDR, ZlibHeader, downcast_chunkdata}, deflate::{BlockType, decode_length, decode_distance}}};
use num_enum::{FromPrimitive, TryFromPrimitive};

mod chunks;
pub use chunks::{ChunkType, ChunkData};

mod readers;
pub use readers::ChunkReader;

mod checksum;
pub use checksum::{CRC32, Adler32};

mod deflate;

mod post_process;

const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const BUFFER_SIZE: usize = 32 * 1024; // 32 KiB

#[repr(u8)]
#[derive(Debug, TryFromPrimitive, Clone, Copy)]
enum ColorType {
    Grayscale = 0,
    Truecolor = 2,
    Indexed = 3,
    GrayscaleAlpha = 4,
    TruecolorAlpha = 6,
}
#[allow(clippy::from_over_into)]
impl Into<PixelFormat> for ColorType {
    fn into(self) -> PixelFormat {
        match self {
            ColorType::Grayscale => PixelFormat::Grayscale,
            ColorType::Truecolor => PixelFormat::Truecolor,
            ColorType::Indexed => todo!(),
            ColorType::GrayscaleAlpha => PixelFormat::GrayscaleAlpha,
            ColorType::TruecolorAlpha => PixelFormat::TruecolorAlpha,
        }
    }
}

#[derive(Debug, TryFromPrimitive, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum PngFilter {
    None = 0,
    Sub = 1,
    Up = 2,
    Average = 3,
    Paeth = 4
}

#[derive(Debug)]
pub struct PngDecoder<'a, R: BufRead, const D: u8, const F: u8> {
    reader: ChunkReader<R>,
    image_buffer: HistoryBuffer<u8>,
    deflate_buffer: Option<HistoryBuffer<u8>>,
    scanline_buffer: HistoryBuffer<u8>,
    phantom: std::marker::PhantomData<&'a ()>,
    ihdr: IHDR,
    cur_block: deflate::Block,
    cur_scanline: (PngFilter, usize), // (filter, bytes remaining)
    source_bitspp: u8,
    source_bytespp: u8
}

impl<'a, R: BufRead, const D: u8, const F: u8> ImageDecoder<'a, R, D, F> for PngDecoder<'a, R, D, F> {
    fn open(mut reader: R) -> Result<Self, DecodingError> {
        check_header(&mut reader)?;

        let mut reader = ChunkReader::new(reader);

        let ihdr = read_ihdr(&mut reader)?;

        let source_bitspp = PixelFormat::from(ihdr.color_type.into()) as u8 * ihdr.bit_depth;
        let mut decoder = Self {
            reader,
            image_buffer: HistoryBuffer::new(BUFFER_SIZE),
            deflate_buffer: None,
            scanline_buffer: HistoryBuffer::new(((ihdr.width as usize + 2) * source_bitspp as usize) / 8), // + 2 for upperleft
            phantom: std::marker::PhantomData,
            ihdr,
            cur_block: deflate::Block::default(),
            cur_scanline: (PngFilter::None, 0),
            source_bitspp,
            source_bytespp: source_bitspp / 8
        };

        loop  {
            decoder.reader.open_chunk()?;

            if decoder.reader.cur_type() == ChunkType::Iend {
                return Err(DecodingError::NoIDAT);
            }

            decoder.update_with_chunk()?;

            if decoder.reader.cur_type() == ChunkType::Idat {
                break; // update_with_chunk has called prepare_for_decompression here.
            }
        }

        decoder.cur_block.load_block(&mut decoder.reader)?;

        Ok(decoder)
    }

    fn fill_buf(&mut self) -> Result<&[u8], DecodingError> {
        match self.cur_block.r#type {
            BlockType::Uncompressed(len) => {
                let fill_len = (len as usize).min(self.remaining_space());

                for _ in 0..fill_len {
                    let b = self.reader.fill_buf()?[0];
                    self.reader.consume(1);
                    self.emit_inflated_byte(b)?;
                }

                if len == fill_len as u16 {
                    self.next_block()?;
                } else {
                    self.cur_block.r#type = BlockType::Uncompressed(len - fill_len as u16);
                }
            }
            BlockType::CompressedFixed => {self.fill_buf_compressed::<true>()?;},
            BlockType::CompressedDynamic => {self.fill_buf_compressed::<false>()?;},
            BlockType::Finished => {
                for _ in (0..self.deflate_buffer().len().min(self.remaining_space() - self.deflate_buffer().remaining_space())).rev() {
                    let b = self.deflate_buffer()[self.deflate_buffer().len() - 1];
                    self.push_inflated_byte(b)?;
                    self.deflate_buffer_mut().consume(1);
                }

                for i in (0..self.scanline_buffer.len().min(self.image_buffer.remaining_space())).rev() {
                    let b = self.scanline_buffer[i];
                    self.push_filtered_byte(b)?;
                    self.scanline_buffer.consume(1);
                }
            }
        }

        if self.cur_block.r#type != BlockType::Finished && self.image_buffer.len() == 0 {
            return self.fill_buf();
        }

        Ok(self.image_buffer.get_first_slice())
    }

    fn consume(&mut self, amt: usize) {
        self.image_buffer.consume(amt);
    }

    fn bit_depth(&self) -> u8 {self.ihdr.bit_depth}

    fn pixel_format(&self) -> crate::PixelFormat {self.ihdr.color_type.into()}
}

impl<'a, R: BufRead, const D: u8, const F: u8> PngDecoder<'a, R, D, F> {
    fn next_block(&mut self) -> Result<(), DecodingError> {
        if self.reader.cur_type() != ChunkType::Idat {
            return Err(DecodingError::InvalidChunk(self.reader.cur_type()));
        }

        if self.cur_block.last {
            return self.finish_decoding();
        }

        self.cur_block.load_block(&mut self.reader)?;

        Ok(())
    }

    fn finish_decoding(&mut self) -> Result<(), DecodingError> {
        self.reader.validate_adler32()?;

        self.reader.close_chunk()?;

        while self.reader.cur_type() != ChunkType::Iend {
            self.reader.open_chunk()?;
            self.update_with_chunk()?;
        }

        self.cur_block.r#type = BlockType::Finished;

        Ok(())
    }

    fn emit_inflated_byte(&mut self, b: u8) -> Result<(), DecodingError> {
        self.reader.update_adler32(b);

        if self.deflate_buffer().remaining_space() == 0 {
            let b = self.deflate_buffer()[self.deflate_buffer().buffer.len() - 1];
            self.push_inflated_byte(b)?;
            self.deflate_buffer_mut().consume(1);
        }

        self.deflate_buffer_mut().push(b);

        Ok(())
    }

    fn deflate_buffer(&self) -> &HistoryBuffer<u8> {
        self.deflate_buffer.as_ref().unwrap_or_else(|| panic!("responsibility of caller to ensure this is only called after start of data reading."))
    }

    fn deflate_buffer_mut(&mut self) -> &mut HistoryBuffer<u8> {
        self.deflate_buffer.as_mut().unwrap_or_else(|| panic!("responsibility of caller to ensure this is only called after start of data reading."))
    }

    fn push_inflated_byte(&mut self, b: u8) -> Result<(), DecodingError> {
        if let Some(corrected) = self.filter(b)? {
            if self.scanline_buffer.remaining_space() == 0 {
                let b = self.scanline_buffer[self.scanline_buffer.buffer.len() - 1];
                self.push_filtered_byte(b)?;
                self.scanline_buffer.consume(1);
            }

            self.scanline_buffer.push(corrected);
        }

        Ok(())
    }

    fn push_filtered_byte(&mut self, b: u8) -> Result<(), DecodingError> {
        self.image_buffer.push(b);

        Ok(())
    }

    /// how many deflate bytes can be emitted before the image buffer is full.
    fn remaining_space(&self) -> usize {
        if true {
            return self.image_buffer.remaining_space() + self.deflate_buffer().remaining_space() + self.scanline_buffer.remaining_space();
        }

        self.image_buffer.remaining_space() * 8 / self.dest_bitspp() as usize * self.source_bitspp as usize
    }

    fn update_with_chunk(&mut self) -> Result<(), DecodingError> {
        if matches!(self.reader.cur_type(), ChunkType::UnkownAncillerary | ChunkType::Iend) {
            return Ok(());
        }

        let chunk_data = self.reader.read_data()?;

        if let Err(err) = chunk_data.validate() {
            if self.reader.cur_type().is_critical() {
                return Err(err);
            }
            return Ok(());
        }

        match self.reader.cur_type() {
            ChunkType::UnkownAncillerary | ChunkType::Iend => unreachable!(),
            ChunkType::Ihdr => Err(DecodingError::MultipleChunks(ChunkType::Ihdr)),
            ChunkType::Idat => downcast_chunkdata::<ZlibHeader>(chunk_data).unwrap().update_decoder(self),
            _ => todo!()
        }
    }

    fn fill_buf_compressed<const S: bool>(&mut self) -> Result<(), DecodingError> {
        loop  {
            if self.remaining_space() < 256 {
                break;
            }

            let (litlen_tree, distance_tree) = if S {
                todo!()
            } else {
                (&self.cur_block.litlen_tree, &self.cur_block.distance_tree)
            };

            let symbol = litlen_tree.decode_symbol(&mut self.reader)?;

            if symbol < 256 {
                self.emit_inflated_byte(symbol as u8)?;
            } else if symbol == 256 {
                self.next_block()?;
                break;
            } else {
                let length = decode_length(symbol, &mut self.reader)?;
                let dist_code = distance_tree.decode_symbol(&mut self.reader)?;
                let distance = decode_distance(dist_code, &mut self.reader)?;

                for _ in 0..length {
                    let byte = self.deflate_buffer()[distance as usize - 1];
                    self.emit_inflated_byte(byte)?;
                }
            }
        } Ok(())
    }
}

fn check_header<R: Read>(data: &mut R) -> Result<(), DecodingError> {
    let header = crate::read_exact_array::<8,_>(data)?;
    if header != PNG_HEADER {
        return Err(DecodingError::InccorectHeader(header.to_vec()))
    }
    Ok(())
}

fn read_ihdr<R: BufRead>(reader: &mut ChunkReader<R>) -> Result<IHDR, DecodingError> {
    reader.open_chunk()?;

    if reader.cur_type() != ChunkType::Ihdr {
        return Err(DecodingError::NoIHDR(reader.cur_type()));
    }

    let ihdr = *downcast_chunkdata::<IHDR>(reader.read_data()?).unwrap();
    ihdr.validate()?;

    Ok(ihdr)
}
