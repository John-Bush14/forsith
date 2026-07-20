use std::io::{BufRead, Read};
use crate::{Channel, CursorVec, DecodingError, OutputWriter, ImageDecoder, PixelFormat, png::{chunks::{ColorPalette, Ihdr, ZlibHeader, tRNS}, deflate::{BlockType, STATIC_DISTANCE_TREE, STATIC_LITLEN_TREE, decode_distance, decode_length}, postprocessing::PostProcessor}};
use num_enum::TryFromPrimitive;

mod chunks;
pub use chunks::{ChunkType, ChunkData};

mod reader;
pub use reader::PngReader;

mod checksum;
pub use checksum::CRC32;

mod deflate;

mod postprocessing;

mod simd;

#[cfg(test)]
mod pngsuite;


const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

#[repr(u8)]
#[derive(Debug, TryFromPrimitive, Clone, Copy, PartialEq, Eq)]
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
            ColorType::Indexed => PixelFormat::Truecolor,
            ColorType::GrayscaleAlpha => PixelFormat::GrayscaleAlpha,
            ColorType::TruecolorAlpha => PixelFormat::TruecolorAlpha,
        }
    }
}

#[derive(Debug)]
pub struct PngDecoder<'a, R: BufRead, C: Channel, const F: u8> {
    reader: PngReader<R>,
    deflate_buffer: CursorVec<u8>,
    scanline_multiples: usize,
    postprocessor: PostProcessor<F>,
    phantom: std::marker::PhantomData<&'a C>,
    ihdr: Ihdr,
    cur_block: deflate::Block,
    inflate_capacity: usize,
    deflate_buffer_tail: usize,
    done: bool
}

impl<'a, R: BufRead, C: Channel, const F: u8> ImageDecoder<'a, R, C, F> for PngDecoder<'a, R, C, F> {
    fn open_validated(mut reader: R) -> Result<Self, DecodingError> {
        check_header(&mut reader)?;

        let mut reader = PngReader::new(reader)?;

        let ihdr = read_ihdr(&mut reader)?;

        println!("{ihdr:?}");
        let mut decoder = Self {
            reader,
            deflate_buffer: CursorVec::new(0),
            scanline_multiples: 0,
            phantom: std::marker::PhantomData,
            postprocessor: PostProcessor::new::<C>(ihdr.width, ihdr.color_type, ihdr.channel_depth),
            ihdr,
            cur_block: deflate::Block::default(),
            inflate_capacity: 0,
            deflate_buffer_tail: 0,
            done: false
        };

        loop  {
            decoder.reader.open_chunk()?;

            if decoder.reader.cur_chunk_type() == ChunkType::Iend {
                return Err(DecodingError::NoIDAT);
            }

            decoder.update_with_chunk()?;

            if decoder.reader.cur_chunk_type() == ChunkType::Idat {
                break; // update_with_chunk has called prepare_for_decompression here.
            }
        }

        if decoder.postprocessor.palette().is_none() && decoder.ihdr.color_type == ColorType::Indexed {
            return Err(DecodingError::NoPallete);
        }

        decoder.cur_block.load_block(&mut decoder.reader)?;

        Ok(decoder)
    }

    fn read(&mut self, dest: &mut [u8]) -> Result<usize, DecodingError> {
        let mut dest = OutputWriter::new(dest);
        self.inflate_capacity = self.calculate_inflate_capacity(&mut dest);

        match self.cur_block.r#type {
            BlockType::Uncompressed(len) => {
                let fill_len = (len as usize).min(self.inflate_capacity());

                for _ in 0..fill_len {
                    let b = self.reader.buffer.read_be::<u8>();
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
                self.decrease_inflate_capacity(self.deflate_buffer.capacity() - self.deflate_buffer.len());

                while self.inflate_capacity() >= self.scanline_bytes() - 1 && self.deflate_buffer.len() - self.deflate_buffer_tail >= self.scanline_bytes() {
                    let scanline = self.deflate_buffer.slice(self.deflate_buffer_tail..self.deflate_buffer_tail + self.scanline_bytes());

                    self.postprocessor.consume_inflated_scanline(scanline, &mut dest)?;

                    self.deflate_buffer_tail += self.scanline_bytes();

                    self.decrease_inflate_capacity(self.scanline_pixel_bytes());
                }

                self.decrease_inflate_capacity(self.postprocessor.remaining_bytes());

                while self.inflate_capacity() >= self.postprocessor.prev_buffer().len() && !self.postprocessor.prev_buffer().is_empty(){
                    self.postprocessor.drain_previous_scanline(&mut dest)?;

                    self.postprocessor.switch_buffers();
                }

                self.done = self.deflate_buffer.is_empty() && self.postprocessor.is_empty();

                if !self.done {dest.set_full();}
            }
        }

        if !dest.is_full() && !self.done {
            let filled_len = dest.len();
            return Ok(filled_len + self.read(dest.remaining_mut_slice())?)
        }

        Ok(dest.len())
    }

    fn bit_depth(&self) -> u8 {self.ihdr.channel_depth}

    fn pixel_format(&self) -> crate::PixelFormat {self.ihdr.color_type.into()}
}

impl<'a, R: BufRead, C: Channel, const F: u8> PngDecoder<'a, R, C, F> {
    fn calculate_inflate_capacity(&mut self, dest: &mut OutputWriter) -> usize {
        // let dest_bitspp = self.dest_bitspp();

        // let correct_dest_capacity = (dest.len() * 8 / dest_bitspp as usize) * self.source_bitspp as usize;
        let correct_dest_capacity = dest.capacity();

        correct_dest_capacity + self.deflate_buffer.remaining() + self.postprocessor.remaining_bytes()
    }

    fn decrease_inflate_capacity(&mut self, change: usize) {unsafe {
        self.inflate_capacity = self.inflate_capacity.unchecked_sub(change);
    }}

    fn next_block(&mut self) -> Result<(), DecodingError> {
        if self.reader.cur_chunk_type() != ChunkType::Idat {
            return Err(DecodingError::InvalidChunk(self.reader.cur_chunk_type()));
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

        while self.reader.cur_chunk_type() != ChunkType::Iend {
            self.reader.open_chunk()?;
            self.update_with_chunk()?;
        }

        self.cur_block.r#type = BlockType::Finished;

        Ok(())
    }

    fn emit_backreferenced_inflated_bytes(&mut self, length: usize, distance: usize, dest: &mut OutputWriter) -> Result<(), DecodingError> {
        let mut remaining = length;
        let start = self.deflate_buffer.len() - distance;

        while remaining > 0 {
            let len = remaining.min(self.deflate_buffer.remaining());

            let cur_start = length - remaining + start;
            self.deflate_buffer.copy_within(cur_start..cur_start+len, self.deflate_buffer.len());
            self.deflate_buffer.advance(len);

            self.decrease_inflate_capacity(len);
            remaining -= len;

            if self.deflate_buffer.is_full() {
                self.drain_deflate_buffer(dest)?;
            }
        }

        Ok(())
    }

    fn drain_deflate_buffer(&mut self, dest: &mut OutputWriter) -> Result<(), DecodingError> {
        let mut start = 0;
        for _ in 0..self.scanline_multiples.min(self.inflate_capacity() / self.scanline_bytes()) {
            self.postprocessor.consume_inflated_scanline(self.deflate_buffer.slice(start..start+self.scanline_bytes()), dest)?;
            start += self.scanline_bytes();
        }

        self.reader.update_adler32(self.deflate_buffer.slice(0..start));

        self.deflate_buffer.shift(start);

        Ok(())
    }

    fn emit_inflated_byte(&mut self, b: u8, dest: &mut OutputWriter) -> Result<(), DecodingError> {
        if self.deflate_buffer.len() == self.deflate_buffer.capacity() {
            self.drain_deflate_buffer(dest)?;
        }

        self.decrease_inflate_capacity(1);

        self.deflate_buffer.push(b);

        Ok(())
    }

    /// how many deflate bytes can be emitted before the image buffer is full.
    fn inflate_capacity(&self) -> usize {self.inflate_capacity}

    fn update_with_chunk(&mut self) -> Result<(), DecodingError> {
        let result = match self.reader.cur_chunk_type() {
            ChunkType::UnkownAncillerary | ChunkType::Iend => return Ok(()),
            ChunkType::Ihdr => Err(DecodingError::MultipleChunks(ChunkType::Ihdr)),
            ChunkType::Idat => ZlibHeader::update_decoder(self),
            ChunkType::Plte => ColorPalette::update_decoder(self),
            ChunkType::tRNS => tRNS::update_decoder(self)
        };

        if let Err(err) = result
            && (self.reader.cur_chunk_type().is_critical() || matches!(err, DecodingError::IOError(_)))
        {
            return Err(err);
        }

        Ok(())
    }

    fn read_compressed_chunk<const STATIC: bool>(&mut self, dest: &mut OutputWriter) -> Result<(), DecodingError> {
        loop  {
            if self.inflate_capacity() < self.scanline_pixel_bytes() + 258 {
                dest.set_full();
                break;
            }

            let (litlen_tree, distance_tree) = if STATIC {
                (&STATIC_LITLEN_TREE, &STATIC_DISTANCE_TREE)
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

                if distance as usize > self.deflate_buffer.len() {
                    self.emit_backreferenced_inflated_bytes(length as usize, distance as usize, dest)?;
                } else {
                    for _ in 0..length {
                        let byte = self.deflate_buffer[self.deflate_buffer.len() - distance as usize];
                        self.emit_inflated_byte(byte, dest)?;
                    }
                }
            }
        } Ok(())
    }
}

fn check_header<R: Read>(reader: &mut R) -> Result<(), DecodingError> {
    let mut header = [0u8; 8];
    reader.read_exact(&mut header)?;
    if header != PNG_HEADER {
        println!("{PNG_HEADER:?}");
        return Err(DecodingError::InccorectHeader(header.to_vec()))
    }
    Ok(())
}

fn read_ihdr<R: BufRead>(reader: &mut PngReader<R>) -> Result<Ihdr, DecodingError> {
    reader.open_chunk()?;

    if reader.cur_chunk_type() != ChunkType::Ihdr {
        return Err(DecodingError::NoIHDR(reader.cur_chunk_type()));
    }

    let ihdr = Ihdr::read(reader, reader.cur_chunk_len())?;
    ihdr.validate()?;

    Ok(ihdr)
}
