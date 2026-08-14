use std::io::Read;
use crate::{Channel, DecodingError, ImageDecoder, PixelFormat, bitspp, buffers::{BitCursorVec, CursorVec, OutputWriter}, parsing::SegmentParser, png::{checksums::Adler32, chunks::{ColorPalette, Ihdr, ZlibDataStream, tRNS}, deflate::{BlockType, MAX_BACKREF_LEN, STATIC_DISTANCE_TREE, STATIC_LITLEN_TREE, decode_distance, decode_length}, postprocessing::{MAX_STRIDE, PostProcessor}}};
use derive_more::IsVariant;
use num_enum::TryFromPrimitive;

mod chunks;
pub use chunks::{ChunkType, ChunkData};

mod parser;
pub use parser::{ChunkParser, ChunkHeader};

use crate::checksums;

mod deflate;

mod postprocessing;

const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];


#[repr(u8)]
#[derive(Debug, TryFromPrimitive, Clone, Copy, PartialEq, Eq, IsVariant)]
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
            ColorType::Grayscale => Self::Grayscale,
            ColorType::Truecolor | ColorType::Indexed => Self::Truecolor,
            ColorType::GrayscaleAlpha => Self::GrayscaleAlpha,
            ColorType::TruecolorAlpha => Self::TruecolorAlpha,
        }
    }
}

#[derive(Debug)]
pub struct PngDecoder<'a, C: Channel, const F: u8> {
    compressed_data: BitCursorVec,
    deflate_buffer: CursorVec<u8>,
    scanline_multiples: usize,
    postprocessor: PostProcessor<C, F>,
    phantom: std::marker::PhantomData<&'a C>,
    ihdr: Ihdr,
    cur_block: deflate::Block,
    deflate_buffer_tail: usize,
    last_adler_update_i: usize,
    adler: Adler32,
}

impl<'a, C: Channel, const F: u8> ImageDecoder<'a, C, F> for PngDecoder<'a, C, F> {
    fn open_validated<R: Read>(mut reader: R) -> Result<Self, DecodingError> {
        check_header(&mut reader)?;

        let mut chunk_parser = ChunkParser::new(reader);

        let ihdr = read_ihdr(&mut chunk_parser)?;

        let postprocessor = PostProcessor::new(ihdr.width, ihdr.color_type, ihdr.channel_depth);

        let mut decoder = Self {
            compressed_data: BitCursorVec::default(),
            deflate_buffer: CursorVec::default(),
            scanline_multiples: 0,
            phantom: std::marker::PhantomData,
            postprocessor,
            ihdr,
            cur_block: deflate::Block::default(),
            deflate_buffer_tail: 0,
            last_adler_update_i: 0,
            adler: Adler32::default()
        };

        chunk_parser.parse_chunks(|h, d, ()| decoder.update_with_chunk(h, d))?;

        if decoder.postprocessor.palette().is_none() && decoder.ihdr.color_type.is_indexed() {
            return Err(DecodingError::NoPallete);
        }

        if decoder.deflate_buffer.get_ref().is_empty() {
            return Err(DecodingError::NoIDAT);
        }

        decoder.postprocessor.setup_interlacing(&decoder.ihdr);
        decoder.cur_block.load_block(&mut decoder.compressed_data)?;

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
                    while self.can_drain_scanline(&dest) && self.deflate_buffer.cursor() - self.deflate_buffer_tail >= self.scanline_bytes() {
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

impl<C: Channel, const F: u8> PngDecoder<'_, C, F> {
    fn can_drain_scanline(&self, dest: &OutputWriter<'_, C, F>) -> bool {
        dest.remaining_bytes()*8 / bitspp::<C, F>() as usize * self.stored_bpp() >= self.scanline_pixel_bytes()*8 || self.ihdr.interlace_method == 1
    }

    fn stored_bpp(&self) -> usize {self.postprocessor.stored_bpp()}
    fn stored_format(&self) -> PixelFormat {self.postprocessor.stored_format()}
    const fn stored_channel_depth(&self) -> u8 {self.postprocessor.stored_channel_depth()}

    #[cold]
    fn next_block(&mut self) -> Result<(), DecodingError> {
        if self.cur_block.last {
            return self.finish_decoding();
        }

        self.cur_block.load_block(&mut self.compressed_data)?;

        Ok(())
    }

    #[cold]
    fn finish_decoding(&mut self) -> Result<(), DecodingError> {
        self.update_adler();

        self.compressed_data.unconsume_bitbuf();
        self.adler.validate(self.compressed_data.read_be::<u32>()?)?;

        self.cur_block.r#type = BlockType::Finished;

        Ok(())
    }

    #[inline]
    fn emit_backreferenced_inflated_bytes(&mut self, length: usize, distance: usize) {
        let i = self.deflate_buffer.cursor();

        self.deflate_buffer.get_mut().copy_within(i-distance..i-distance+length, i);

        self.deflate_buffer.consume(length);
    }

    fn update_adler(&mut self) {
        self.adler.update(&self.deflate_buffer.get_ref()[self.last_adler_update_i .. self.deflate_buffer.cursor()]);
        self.last_adler_update_i = self.deflate_buffer.cursor();
    }

    #[cold]
    fn drain_deflate_buffer(&mut self, dest: &mut OutputWriter<'_, C, F>) -> Result<(), DecodingError> {
        self.update_adler();

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
        let range = self.deflate_buffer_tail + bytes - prev_scanline..self.deflate_buffer.cursor();

        self.last_adler_update_i -= self.deflate_buffer.cursor() - range.len();

        self.deflate_buffer.set_cursor(range.len());
        self.deflate_buffer.get_mut().copy_within(range, 0);
    }

    #[inline(always)]
    fn consume_inflated_scanline(&mut self, start: usize, dest: &mut OutputWriter<'_, C, F>) -> Result<usize, DecodingError> {
        let prev_scanline_bytes = self.max_scanline_bytes() + MAX_STRIDE;
        let scanline_bytes = self.scanline_bytes();

        let prev_and_cur_scanline = &mut self.deflate_buffer.get_mut()[start - prev_scanline_bytes..start + scanline_bytes];

        self.postprocessor.filter_inflated_scanline(prev_and_cur_scanline, prev_scanline_bytes, dest)?;

        Ok(scanline_bytes)
    }

    pub fn update_with_chunk(&mut self, chunk_header: &ChunkHeader, chunk_data: &mut CursorVec<u8>) -> Result<(), DecodingError> {
        let result = match chunk_header.r#type() {
            ChunkType::Iend | ChunkType::UnkownAncillerary => {chunk_data.consume(chunk_header.len()); Ok(())},
            ChunkType::Ihdr => Err(DecodingError::MultipleChunks(ChunkType::Ihdr)),
            ChunkType::Idat => ZlibDataStream::update_decoder(self, chunk_header, chunk_data),
            ChunkType::Plte => ColorPalette::update_decoder(self, chunk_header, chunk_data),
            ChunkType::tRNS => tRNS::update_decoder(self, chunk_header, chunk_data),
        };

        if let Err(err) = result
            && (chunk_header.is_critical() || matches!(err, DecodingError::IOError(_)))
        {
            return Err(err);
        }

        Ok(())
    }

    #[cold]
    fn read_uncompressed_chunk(&mut self, len: u16, dest: &mut OutputWriter<'_, C, F>) -> Result<(), DecodingError> {
        self.compressed_data.unconsume_bitbuf();

        let mut decoded_bytes = 0;

        loop {
            let chunk_len = (len - decoded_bytes).min(self.deflate_buffer.remaining().saturating_truncate());

            self.deflate_buffer.read_from(&mut **self.compressed_data, chunk_len as usize)?;

            decoded_bytes += chunk_len;

            if decoded_bytes >= len || !self.can_drain_scanline(dest) {
                break;
            }

            self.drain_deflate_buffer(dest)?;
        }

        if len == decoded_bytes {
            self.compressed_data.align()?;
            self.next_block()?;
        } else {
            self.cur_block.r#type = BlockType::Uncompressed(len - decoded_bytes);
            dest.set_full();
        } Ok(())
    }

    #[cold]
    fn read_compressed_chunk<const STATIC: bool>(&mut self, dest: &mut OutputWriter<'_, C, F>) -> Result<(), DecodingError> {
        loop  {
            if std::hint::unlikely(self.deflate_buffer.cursor() + MAX_BACKREF_LEN >= self.deflate_buffer.capacity()) {
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

            let symbol = litlen_tree.decode_symbol(&mut self.compressed_data);

            if symbol < 256 {
                self.deflate_buffer.write_fast_single(u8::try_from(symbol).unwrap());
            } else if std::hint::unlikely(symbol == 256) {
                self.next_block()?;
                break;
            } else {
                let length = decode_length(symbol, &mut self.compressed_data);
                let dist_code = distance_tree.decode_symbol(&mut self.compressed_data);
                let distance = decode_distance(dist_code, &mut self.compressed_data);

                if distance as usize >= length as usize {
                    self.emit_backreferenced_inflated_bytes(length as usize, distance as usize);
                }
                else if distance == 1 {
                    let i = self.deflate_buffer.cursor();

                    let fill = self.deflate_buffer.get_ref()[i - 1];
                    self.deflate_buffer.take_mut_slice(length as usize).fill(fill);
                }
                else {
                    for _ in 0..length-distance {
                        let byte = self.deflate_buffer.get_ref()[self.deflate_buffer.cursor() - distance as usize];
                        self.deflate_buffer.write_fast_single(byte);
                    }

                    self.emit_backreferenced_inflated_bytes(distance as usize, distance as usize);
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

fn read_ihdr<R: Read>(parser: &mut ChunkParser<R>) -> Result<Ihdr, DecodingError> {
    let (chunk_header, mut chunk_data) = parser.parse_first_chunk()?;

    if !chunk_header.is_ihdr() {
        return Err(DecodingError::NoIHDR(chunk_header.r#type()));
    }

    let ihdr = Ihdr::read(&mut chunk_data, chunk_header.len())?;
    ihdr.validate()?;

    Ok(ihdr)
}
