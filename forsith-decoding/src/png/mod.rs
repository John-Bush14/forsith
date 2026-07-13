use std::io::{BufRead, Read};
use crate::{CursorVec, DecodingError, DestinationBuffer, ImageDecoder, PixelFormat, png::{chunks::{IHDR, ZlibHeader, downcast_chunkdata}, deflate::{BlockType, decode_distance, decode_length}, filtering::{Filterer, calculate_scanline_bytes}}};
use num_enum::TryFromPrimitive;

mod chunks;
pub use chunks::{ChunkType, ChunkData};

mod reader;
pub use reader::Reader;

mod checksum;
pub use checksum::CRC32;

mod deflate;

mod filtering;

mod simd;

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
impl From<ColorType> for PixelFormat {
    fn from(f: ColorType) -> Self {
        match f {
            ColorType::Grayscale => PixelFormat::Grayscale,
            ColorType::Truecolor => PixelFormat::Truecolor,
            ColorType::Indexed => todo!(),
            ColorType::GrayscaleAlpha => PixelFormat::GrayscaleAlpha,
            ColorType::TruecolorAlpha => PixelFormat::TruecolorAlpha,
        }
    }
}

#[derive(Debug)]
pub struct PngDecoder<'a, R: BufRead, const D: u8, const F: u8> {
    reader: Reader<R>,
    deflate_buffer: CursorVec<u8>,
    scanline_multiples: usize,
    filterer: Filterer,
    phantom: std::marker::PhantomData<&'a ()>,
    ihdr: IHDR,
    cur_block: deflate::Block,
    _source_bitspp: u8,
    _source_bytespp: u8,
    inflate_capacity: usize,
    deflate_buffer_tail: usize,
}

impl<'a, R: BufRead, const D: u8, const F: u8> ImageDecoder<'a, R, D, F> for PngDecoder<'a, R, D, F> {
    fn open(mut reader: R) -> Result<Self, DecodingError> {
        check_header(&mut reader)?;

        let mut reader = Reader::new(reader);

        let ihdr = read_ihdr(&mut reader)?;

        let source_bitspp = PixelFormat::from(ihdr.color_type) as u8 * ihdr.bit_depth;
        let mut decoder = Self {
            reader,
            deflate_buffer: CursorVec::new(0),
            scanline_multiples: 0,
            phantom: std::marker::PhantomData,
            filterer: Filterer::new(calculate_scanline_bytes(ihdr.width, source_bitspp), source_bitspp as usize / 8),
            ihdr,
            cur_block: deflate::Block::default(),
            _source_bitspp: source_bitspp,
            _source_bytespp: source_bitspp / 8,
            inflate_capacity: 0,
            deflate_buffer_tail: 0,
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
        let mut dest = DestinationBuffer::<D, F>::new(dest);
        self.inflate_capacity = self.calculate_inflate_capacity(&mut dest);

        match self.cur_block.r#type {
            BlockType::Uncompressed(len) => {
                let fill_len = (len as usize).min(self.inflate_capacity());

                for _ in 0..fill_len {
                    let b = self.reader.fill_buf()?[0];
                    self.reader.consume(1);
                    self.emit_inflated_byte(b, &mut dest)?;
                }

                if len == fill_len as u16 {
                    self.next_block()?;
                } else {
                    self.cur_block.r#type = BlockType::Uncompressed(len - fill_len as u16);
                    dest.set_full();
                }
            }
            BlockType::CompressedFixed => {self.read_compressed_chunk::<true>(&mut dest)?;},
            BlockType::CompressedDynamic => {self.read_compressed_chunk::<false>(&mut dest)?;},
            BlockType::Finished => {
                self.update_inflate_capacity(-((self.deflate_buffer.capacity() - self.deflate_buffer.len()) as isize));

                while self.inflate_capacity() >= self.scanline_bytes() - 1 && self.deflate_buffer.len() - self.deflate_buffer_tail >= self.scanline_bytes() {
                    let scanline = self.deflate_buffer.slice(self.deflate_buffer_tail..self.deflate_buffer_tail + self.scanline_bytes());

                    self.filterer.consume_inflated_scanline(scanline, &mut dest)?;

                    self.deflate_buffer_tail += self.scanline_bytes();

                    self.update_inflate_capacity(-(self.scanline_pixel_bytes() as isize));
                }

                self.update_inflate_capacity(-(self.filterer.remaining_bytes() as isize));

                while self.inflate_capacity() >= self.filterer.prev_buffer().len() && !self.filterer.prev_buffer().is_empty(){
                    self.filterer.drain_previous_scanline(&mut dest)?;

                    self.filterer.switch_buffers();
                }
            }
        }

        if !dest.is_full() && self.cur_block.r#type != BlockType::Finished {
            let filled_len = dest.len();
            return Ok(filled_len + self.read(&mut dest.buffer[filled_len..])?)
        }

        Ok(dest.len())
    }

    fn bit_depth(&self) -> u8 {self.ihdr.bit_depth}

    fn pixel_format(&self) -> crate::PixelFormat {self.ihdr.color_type.into()}
}

impl<'a, R: BufRead, const D: u8, const F: u8> PngDecoder<'a, R, D, F> {
    fn calculate_inflate_capacity(&mut self, dest: &mut DestinationBuffer<'_, D, F>) -> usize {
        // let dest_bitspp = self.dest_bitspp();

        // let correct_dest_capacity = (dest.len() * 8 / dest_bitspp as usize) * self.source_bitspp as usize;
        let correct_dest_capacity = dest.capacity();

        correct_dest_capacity + self.deflate_buffer.remaining() + self.filterer.remaining_bytes()
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
        self.reader.update_adler32(self.deflate_buffer.slice(0..self.deflate_buffer.len()));

        self.reader.validate_adler32()?;

        self.reader.close_chunk()?;

        while self.reader.cur_type() != ChunkType::Iend {
            self.reader.open_chunk()?;
            self.update_with_chunk()?;
        }

        self.cur_block.r#type = BlockType::Finished;

        Ok(())
    }

    fn emit_inflated_byte(&mut self, b: u8, dest: &mut DestinationBuffer<'_, D, F>) -> Result<(), DecodingError> {
        if self.deflate_buffer.len() == self.deflate_buffer.capacity() {
            let mut start = 0;
            for _ in 0..self.scanline_multiples.min(self.inflate_capacity() / self.scanline_bytes()) {
                self.filterer.consume_inflated_scanline(self.deflate_buffer.slice(start..start+self.scanline_bytes()), dest)?;
                start += self.scanline_bytes();
            }

            self.reader.update_adler32(self.deflate_buffer.slice(0..start));

            self.deflate_buffer.shift(start);
        }

        self.update_inflate_capacity(-1);

        self.deflate_buffer.push(b);

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

    fn read_compressed_chunk<const STATIC: bool>(&mut self, dest: &mut DestinationBuffer<'_, D, F>) -> Result<(), DecodingError> {
        loop  {
            if self.inflate_capacity() < self.scanline_pixel_bytes() + 258 {
                dest.set_full();
                break;
            }

            let (litlen_tree, distance_tree) = if STATIC {
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
                    let byte = self.deflate_buffer[self.deflate_buffer.len() - distance as usize];
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

fn read_ihdr<R: BufRead>(reader: &mut Reader<R>) -> Result<IHDR, DecodingError> {
    reader.open_chunk()?;

    if reader.cur_type() != ChunkType::Ihdr {
        return Err(DecodingError::NoIHDR(reader.cur_type()));
    }

    let ihdr = *downcast_chunkdata::<IHDR>(reader.read_data()?).unwrap();
    ihdr.validate()?;

    Ok(ihdr)
}
