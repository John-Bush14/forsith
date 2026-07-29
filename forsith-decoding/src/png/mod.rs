use std::io::{BufRead, Read};
use crate::{Channel, CursorVec, DecodingError, ImageDecoder, OutputWriter, PixelFormat, bitspp, compression::BitReader, png::{chunks::{ColorPalette, Ihdr, ZlibHeader, tRNS}, deflate::{BlockType, MAX_BACKREF_LEN, STATIC_DISTANCE_TREE, STATIC_LITLEN_TREE, decode_distance, decode_length}, postprocessing::{MAX_STRIDE, PostProcessor}}};
use num_enum::TryFromPrimitive;

mod chunks;
pub use chunks::{ChunkType, ChunkData};

mod reader;
pub use reader::PngReader;

mod checksum;

mod deflate;

mod postprocessing;

mod simd;

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
    postprocessor: PostProcessor<C, F>,
    phantom: std::marker::PhantomData<&'a C>,
    ihdr: Ihdr,
    cur_block: deflate::Block,
    deflate_buffer_tail: usize,
    last_adler_update_i: usize
}

impl<'a, R: BufRead, C: Channel, const F: u8> ImageDecoder<'a, R, C, F> for PngDecoder<'a, R, C, F> {
    fn open_validated(mut reader: R) -> Result<Self, DecodingError> {
        check_header(&mut reader)?;

        let mut reader = PngReader::new(reader)?;
        let ihdr = read_ihdr(&mut reader)?;

        let postprocessor = PostProcessor::new(ihdr.width, ihdr.color_type, ihdr.channel_depth);

        let mut decoder = Self {
            reader,
            deflate_buffer: CursorVec::new(0),
            scanline_multiples: 0,
            phantom: std::marker::PhantomData,
            postprocessor,
            ihdr,
            cur_block: deflate::Block::default(),
            deflate_buffer_tail: 0,
            last_adler_update_i: 0
        };

        decoder.handle_chunks_until_idat()?;
        if decoder.postprocessor.palette().is_none() && decoder.ihdr.color_type == ColorType::Indexed {
            return Err(DecodingError::NoPallete);
        }

        decoder.postprocessor.setup_interlacing(&decoder.ihdr);
        decoder.cur_block.load_block(&mut decoder.reader)?;

        Ok(decoder)
    }

    #[cold]
    fn read(&mut self, dest: &mut [C::StorageType]) -> Result<usize, DecodingError> {
        if dest.len() < self.min_buf_size() {return Err(DecodingError::TinyDestBuf(dest.len()))}

        let mut dest = OutputWriter::new(dest); self.postprocessor.update_dest(&mut dest);

        while !dest.is_full() {
            match self.cur_block.r#type {
                BlockType::Uncompressed(len) => {self.read_uncompressed_chunk(len, &mut dest)?;},
                BlockType::CompressedFixed => {self.read_compressed_chunk::<true>(&mut dest)?;},
                BlockType::CompressedDynamic => {self.read_compressed_chunk::<false>(&mut dest)?;},
                BlockType::Finished => {
                    while self.can_drain_scanline(&mut dest) && self.deflate_buffer.len() - self.deflate_buffer_tail >= self.scanline_bytes() {
                        let consumed_bytes = self.consume_inflated_scanline(self.deflate_buffer_tail, &mut dest)?;

                        self.deflate_buffer_tail += consumed_bytes;
                    }

                    break;
                }
            }
        }

        if self.ihdr.interlace_method == 1 && !dest.is_empty() {
            Ok(self.max_buf_size())
        } else {
            Ok(dest.len())
        }
    }

    fn image_dimensions(&self) -> (usize, usize) {(self.ihdr.width as _, self.ihdr.height as _)}
    fn min_buf_size(&self) -> usize {
        if self.ihdr.interlace_method == 1 {
            return self.max_buf_size();
        }

        let min_inflate_capacity = self.scanline_pixel_bytes().max(MAX_BACKREF_LEN);

        (min_inflate_capacity * 8 * F as usize * 8 / self.stored_bpp()).div_ceil(8)
    }

    fn source_bit_depth(&self) -> u8 {self.stored_channel_depth()}

    fn source_pixel_format(&self) -> crate::PixelFormat {self.stored_format()}
}

impl<'a, R: BufRead, C: Channel, const F: u8> PngDecoder<'a, R, C, F> {
    fn handle_chunks_until_idat(&mut self) -> Result<(), DecodingError> {
        loop  {
            self.reader.open_chunk()?;

            if self.reader.cur_chunk_type() == ChunkType::Iend {
                return Err(DecodingError::NoIDAT);
            }

            self.update_with_chunk()?;

            if self.reader.cur_chunk_type() == ChunkType::Idat {
                break Ok(()); // update_with_chunk has called prepare_for_decompression here.
            }
        }
    }

    fn can_drain_scanline(&self, dest: &mut OutputWriter<'_, C, F>) -> bool {
        dest.remaining_bytes()*8 / bitspp::<C, F>() as usize * self.stored_bpp() >= self.scanline_pixel_bytes()*8 || self.ihdr.interlace_method == 1
    }

    fn stored_bpp(&self) -> usize {self.postprocessor.stored_bpp()}
    fn stored_format(&self) -> PixelFormat {self.postprocessor.stored_format()}
    fn stored_channel_depth(&self) -> u8 {self.postprocessor.stored_channel_depth()}

    #[cold]
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

    #[cold]
    fn finish_decoding(&mut self) -> Result<(), DecodingError> {
        self.reader.update_adler32(self.deflate_buffer.slice(self.last_adler_update_i .. self.deflate_buffer.len()));

        self.reader.unconsume_bitbuf();
        self.reader.validate_adler32()?;

        while self.reader.cur_chunk_type() != ChunkType::Iend {
            self.reader.open_chunk()?;
            self.update_with_chunk()?;
        }

        self.cur_block.r#type = BlockType::Finished;

        Ok(())
    }

    #[inline]
    fn emit_backreferenced_inflated_bytes(&mut self, length: usize, distance: usize) {
        let i = self.deflate_buffer.len();

        self.deflate_buffer.copy_within(i-distance..i-distance+length, i);

        self.deflate_buffer.advance(length);
    }

    #[cold]
    fn drain_deflate_buffer(&mut self, dest: &mut OutputWriter<'_, C, F>) -> Result<(), DecodingError> {
        self.reader.update_adler32(self.deflate_buffer.slice(self.last_adler_update_i..self.deflate_buffer.len()));
        self.last_adler_update_i = self.deflate_buffer.len();

        let mut drained_bytes = 0;

        for _ in 0..self.scanline_multiples {
            if !self.can_drain_scanline(dest) {break}

            drained_bytes += self.consume_inflated_scanline(self.deflate_buffer_tail, dest)?;
        }

        self.deflate_buffer_remove(drained_bytes);

        Ok(())
    }

    fn deflate_buffer_remove(&mut self, bytes: usize) {
        let prev_scanline = self.max_scanline_bytes() + MAX_STRIDE;
        let range = self.deflate_buffer_tail + bytes - prev_scanline..self.deflate_buffer.len();

        self.last_adler_update_i -= self.deflate_buffer.len() - range.len();

        self.deflate_buffer.set_cursor(range.len());
        self.deflate_buffer.copy_within(range, 0);
    }

    #[inline(always)]
    fn consume_inflated_scanline(&mut self, start: usize, dest: &mut OutputWriter<'_, C, F>) -> Result<usize, DecodingError> {
        let prev_scanline_bytes = self.max_scanline_bytes() + MAX_STRIDE;
        let scanline_bytes = self.scanline_bytes();

        let prev_and_cur_scanline = self.deflate_buffer.mut_slice(start - prev_scanline_bytes..start + scanline_bytes);

        self.postprocessor.filter_inflated_scanline(prev_and_cur_scanline, prev_scanline_bytes, dest)?;

        Ok(scanline_bytes)
    }

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

    #[cold]
    fn read_uncompressed_chunk(&mut self, len: u16, dest: &mut OutputWriter<'_, C, F>) -> Result<(), DecodingError> {
        let fill_len = (len as usize).min(self.deflate_buffer.remaining() - self.deflate_buffer_tail + dest.remaining_bytes());
        let mut remaining = fill_len;

        self.reader.buffer.unconsume(self.reader.bit_buf.bits_remaining() as usize / 8);
        self.reader.consume_bits(self.reader.bit_buf.bits_remaining());

        loop {
            let chunk_len = remaining.min(self.deflate_buffer.remaining());

            let i = self.deflate_buffer.cursor;
            self.reader.read_exact(self.deflate_buffer.mut_slice(i..i + chunk_len))?;
            self.deflate_buffer.advance(chunk_len);

            remaining -= chunk_len;

            if remaining == 0 {break;} else {
                self.drain_deflate_buffer(dest)?;
            }
        }

        if len as usize == fill_len {
            self.reader.align()?;

            self.next_block()?;
        } else {
            self.cur_block.r#type = BlockType::Uncompressed(len - fill_len as u16);
            dest.set_full();
        } Ok(())
    }

    #[cold]
    fn read_compressed_chunk<const STATIC: bool>(&mut self, dest: &mut OutputWriter<'_, C, F>) -> Result<(), DecodingError> {
        loop  {
            if std::hint::unlikely(self.deflate_buffer.len() + MAX_BACKREF_LEN >= self.deflate_buffer.capacity()) {
                if self.can_drain_scanline(dest) {
                    self.drain_deflate_buffer(dest)?;
                } else {
                    dest.set_full();
                    break;
                }
            }

            let (litlen_tree, distance_tree) = if STATIC {
                (&STATIC_LITLEN_TREE, &STATIC_DISTANCE_TREE)
            } else {
                (&self.cur_block.litlen_tree, &self.cur_block.distance_tree)
            };

            let symbol = litlen_tree.decode_symbol(&mut self.reader);

            if symbol < 256 {
                self.deflate_buffer.push(symbol as u8);
            } else if std::hint::unlikely(symbol == 256) {
                self.next_block()?;
                break;
            } else {
                let length = decode_length(symbol, &mut self.reader);
                let dist_code = distance_tree.decode_symbol(&mut self.reader);
                let distance = decode_distance(dist_code, &mut self.reader);

                if distance as usize >= length as usize {
                    self.emit_backreferenced_inflated_bytes(length as usize, distance as usize);
                }
                else if distance == 1 {
                    let i = self.deflate_buffer.len();

                    let fill = self.deflate_buffer[i - 1];
                    self.deflate_buffer.buffer[i..i + length as usize].fill(fill);
                    self.deflate_buffer.advance(length as usize);
                }
                else {
                    for _ in 0..length {
                        let byte = self.deflate_buffer[self.deflate_buffer.len() - distance as usize];
                        self.deflate_buffer.push(byte);
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
