use core::panic;
use std::io::{BufRead, Read};
use crate::{DecodingError, HistoryBuffer, ImageDecoder, Num, PixelFormat, png::{chunks::{IHDR, ZlibHeader, downcast_chunkdata}, deflate::{BlockType, decode_distance, decode_length}, post_process::scanline_bytes}};
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
    deflate_buffer: Option<HistoryBuffer<u8>>,
    scanline_buffers: [Vec<u8>; 2],
    cur_scanline_buffer: usize,
    phantom: std::marker::PhantomData<&'a ()>,
    ihdr: IHDR,
    cur_block: deflate::Block,
    source_bitspp: u8,
    source_bytespp: u8,
    inflate_capacity: usize,
    dest_index: usize,
}

impl<'a, R: BufRead, const D: u8, const F: u8> ImageDecoder<'a, R, D, F> for PngDecoder<'a, R, D, F> {
    fn open(mut reader: R) -> Result<Self, DecodingError> {
        check_header(&mut reader)?;

        let mut reader = ChunkReader::new(reader);

        let ihdr = read_ihdr(&mut reader)?;

        let source_bitspp = PixelFormat::from(ihdr.color_type.into()) as u8 * ihdr.bit_depth;
        let scanline_buffer_size = scanline_bytes(ihdr.width, source_bitspp) - 1;
        let mut decoder = Self {
            reader,
            deflate_buffer: None,
            phantom: std::marker::PhantomData,
            scanline_buffers: [Vec::with_capacity(scanline_buffer_size), Vec::with_capacity(scanline_buffer_size)],
            cur_scanline_buffer: 0,
            ihdr,
            cur_block: deflate::Block::default(),
            source_bitspp,
            source_bytespp: source_bitspp / 8,
            inflate_capacity: 0,
            dest_index: 0,
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

    fn read(&mut self, dest: &mut [u8]) -> Result<usize, DecodingError> {
        self.dest_index = 0;
        self.inflate_capacity = self.calculate_inflate_capacity(dest);

        match self.cur_block.r#type {
            BlockType::Uncompressed(len) => {
                let fill_len = (len as usize).min(self.inflate_capacity());

                for _ in 0..fill_len {
                    let b = self.reader.fill_buf()?[0];
                    self.reader.consume(1);
                    self.emit_inflated_byte(b, dest)?;
                }

                if len == fill_len as u16 {
                    self.next_block()?;
                } else {
                    self.cur_block.r#type = BlockType::Uncompressed(len - fill_len as u16);
                }
            }
            BlockType::CompressedFixed => {self.fill_buf_compressed::<true>(dest)?;},
            BlockType::CompressedDynamic => {self.fill_buf_compressed::<false>(dest)?;},
            BlockType::Finished => {
                self.update_inflate_capacity(-(self.deflate_buffer().remaining_space() as isize));

                while self.inflate_capacity() >= self.scanline_bytes() - 1 && self.deflate_buffer().len() > 0 {
                    self.consume_inflated_scanline(dest)?;

                    self.update_inflate_capacity(-(self.scanline_bytes() as isize - 1));
                }

                self.update_inflate_capacity(-(self.remaining_scanline_buffers_bytes() as isize));

                while self.inflate_capacity() >= self.prev_scanline_buffer().len() && !self.prev_scanline_buffer().is_empty(){
                    self.push_previous_scanline(dest)?;

                    self.switch_scanline_buffers();
                }
            }
        }

        if self.cur_block.r#type != BlockType::Finished && self.dest_index == 0 {
            return self.read(dest);
        }

        Ok(self.dest_index)
    }

    fn bit_depth(&self) -> u8 {self.ihdr.bit_depth}

    fn pixel_format(&self) -> crate::PixelFormat {self.ihdr.color_type.into()}
}

impl<'a, R: BufRead, const D: u8, const F: u8> PngDecoder<'a, R, D, F> {
    fn calculate_inflate_capacity(&mut self, dest: &mut [u8]) -> usize {
        // let dest_bitspp = self.dest_bitspp();

        // let correct_dest_capacity = (dest.len() * 8 / dest_bitspp as usize) * self.source_bitspp as usize;
        let correct_dest_capacity = dest.len() - self.dest_index;

        correct_dest_capacity + self.deflate_buffer().remaining_space() + self.remaining_scanline_buffers_bytes()
    }

    fn update_inflate_capacity(&mut self, change: isize) {
        self.inflate_capacity = (self.inflate_capacity as isize + change) as usize;
    }

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

    fn emit_inflated_byte(&mut self, b: u8, dest: &mut [u8]) -> Result<(), DecodingError> {
        self.reader.update_adler32(b);

        if self.deflate_buffer().is_full() {
            println!("consume scanline inflate_capacity: {}, prev_scanline_buffer.len(): {}", self.inflate_capacity(), self.prev_scanline_buffer().len());
            self.consume_inflated_scanline(dest)?;
        }

        self.update_inflate_capacity(-1);

        self.deflate_buffer_mut().push(b);

        Ok(())
    }

    fn deflate_buffer(&self) -> &HistoryBuffer<u8> {
        self.deflate_buffer.as_ref().unwrap_or_else(|| panic!("responsibility of caller to ensure this is only called after start of data reading."))
    }

    fn deflate_buffer_mut(&mut self) -> &mut HistoryBuffer<u8> {
        self.deflate_buffer.as_mut().unwrap_or_else(|| panic!("responsibility of caller to ensure this is only called after start of data reading."))
    }

    fn push_previous_scanline(&mut self, dest: &mut [u8]) -> Result<(), DecodingError> {
        if self.inflate_capacity() < self.prev_scanline_buffer().len() {
            panic!("responsibility of caller to ensure this is only called when there is enough space in the destination buffer.")
        }

        for i in 0..self.prev_scanline_buffer().len() {
            let b = self.prev_scanline_buffer()[i];
            dest[self.dest_index] = b;
            self.dest_index += 1;
        }

        self.prev_scanline_buffer_mut().clear();

        Ok(())
    }

    /// how many deflate bytes can be emitted before the image buffer is full.
    fn inflate_capacity(&self) -> usize {self.inflate_capacity}

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

    fn fill_buf_compressed<const S: bool>(&mut self, dest: &mut [u8]) -> Result<(), DecodingError> {
        loop  {
            if self.inflate_capacity() < self.scanline_bytes() - 1 + 257 + 1000 {
                break;
            }

            let (litlen_tree, distance_tree) = if S {
                todo!()
            } else {
                (&self.cur_block.litlen_tree, &self.cur_block.distance_tree)
            };

            let symbol = litlen_tree.decode_symbol(&mut self.reader)?;

            if symbol < 256 {
                self.emit_inflated_byte(symbol as u8, dest)?;
            } else if symbol == 256 {
                self.next_block()?;
                break;
            } else {
                let length = decode_length(symbol, &mut self.reader)?;
                let dist_code = distance_tree.decode_symbol(&mut self.reader)?;
                let distance = decode_distance(dist_code, &mut self.reader)?;

                for _ in 0..length {
                    let byte = self.deflate_buffer()[distance as usize - 1];
                    self.emit_inflated_byte(byte, dest)?;
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
