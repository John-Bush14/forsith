use crate::{
    DecodingError,
    bit::BitReader,
    buffers::CursorVec,
    checksums,
    decompression::deflate::{self, DeflateDecoder},
    image::{
        ChannelLayout, ImageDecoder, PixelFormat, SampleType,
        outputconverting::OutputWriter,
        png::{
            adam7_pass::Adam7Pass,
            checksums::Adler32,
            chunks::{ColorType, Ihdr},
            deflate::{BlockType, MAX_BACKREF_LEN},
            filtering::InplaceDefilterer,
            output::PngOutputWriter,
        },
    },
    parsing::SegmentParser,
};
use std::io::Read;

mod chunks;
pub use chunks::ChunkType;

mod parser;
pub use parser::ChunkParser;

mod filtering;

mod output;

mod adam7_pass;

const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const MAX_STRIDE: usize = 8;

pub struct PngDecoder<'a> {
    compressed_data: BitReader<CursorVec<u8>>,
    deflate_buffer: CursorVec<u8>,
    scanline_multiples: usize,
    ihdr: Ihdr,
    stride: usize,
    stored_format: PixelFormat,
    deflate_buffer_tail: usize,
    last_adler_update_i: usize,
    adler: Adler32,
    pass: Adam7Pass,
    deflate: DeflateDecoder,
    output: PngOutputWriter,
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> ImageDecoder<'a> for PngDecoder<'a> {
    fn open<R: Read>(
        mut reader: R,
        _pereferred_pixel_format: PixelFormat,
    ) -> Result<Self, DecodingError> {
        check_header(&mut reader)?;

        let mut chunk_parser = ChunkParser::new(reader);

        let ihdr = read_ihdr(&mut chunk_parser)?;
        let stored_format = stored_format(&ihdr);

        let mut decoder = Self {
            compressed_data: BitReader::default(),
            deflate_buffer: CursorVec::default(),
            scanline_multiples: 0,
            output: PngOutputWriter::new(ihdr.color_type, stored_format),
            pass: Adam7Pass::new(&ihdr, stored_format),
            stride: stride(stored_format),
            stored_format,
            ihdr,
            deflate: DeflateDecoder::default(),
            deflate_buffer_tail: 0,
            last_adler_update_i: 0,
            adler: Adler32::default(),
            phantom: std::marker::PhantomData,
        };

        decoder.parse_everything(chunk_parser)?;
        decoder.validate_before_decoding()?;
        decoder.next_block()?;

        Ok(decoder)
    }

    #[cold]
    fn read(&mut self, dest: &mut [u8]) -> Result<usize, DecodingError> {
        if dest.len() < self.min_buf_size() {
            return Err(DecodingError::TinyDestBuf(dest.len()));
        }

        let mut dest = OutputWriter::new(dest, self.output_format());
        self.pass.update_dest(&mut dest);

        while !dest.is_full() {
            match self.deflate.cur_block().ty {
                BlockType::Uncompressed(len) => {
                    self.read_uncompressed_chunk(len, &mut dest)?;
                }
                BlockType::CompressedFixed => {
                    self.read_compressed_chunk::<true>(&mut dest)?;
                }
                BlockType::CompressedDynamic => {
                    self.read_compressed_chunk::<false>(&mut dest)?;
                }
                BlockType::Finished => {
                    while self.can_drain_scanline(&dest)
                        && self.deflate_buffer.cursor() - self.deflate_buffer_tail
                            >= self.scanline_bytes()
                    {
                        let consumed_bytes =
                            self.consume_inflated_scanline(self.deflate_buffer_tail, &mut dest)?;

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

    fn image_dimensions(&self) -> (usize, usize) {
        (self.ihdr.width as _, self.ihdr.height as _)
    }

    fn min_buf_size(&self) -> usize {
        if self.ihdr.interlace_method == 1 {
            return self.max_buf_size();
        }

        let min_inflate_capacity = self.scanline_pixel_bytes().max(MAX_BACKREF_LEN);

        (min_inflate_capacity * 8 * self.output_format().bpp() as usize
            / self.stored_format().bpp() as usize)
            .div_ceil(8)
    }

    fn pixel_format(&self) -> PixelFormat {
        self.stored_format()
    }
}

impl PngDecoder<'_> {
    const fn can_drain_scanline(&self, dest: &OutputWriter<'_>) -> bool {
        dest.remaining() * 8 / self.output_format().bpp() as usize
            * self.stored_format().bpp() as usize
            >= self.scanline_pixel_bytes() * 8
            || self.ihdr.interlace_method == 1
    }

    const fn stored_format(&self) -> PixelFormat {
        self.stored_format
    }

    #[cold]
    fn next_block(&mut self) -> Result<(), DecodingError> {
        if self.deflate.next_block(&mut self.compressed_data)? {
            self.finish_decoding()?;
        }

        Ok(())
    }

    #[cold]
    fn finish_decoding(&mut self) -> Result<(), DecodingError> {
        self.update_adler();

        self.compressed_data.unconsume_bitbuf();
        self.adler.validate(self.compressed_data.read_be()?)?;

        Ok(())
    }

    fn validate_before_decoding(&self) -> Result<(), DecodingError> {
        if self.output.palette().is_none() && self.ihdr.color_type.is_indexed() {
            return Err(DecodingError::NoPallete);
        }

        if self.deflate_buffer.get_ref().is_empty() {
            return Err(DecodingError::NoIDAT);
        }

        Ok(())
    }

    fn update_adler(&mut self) {
        self.adler.update(
            &self.deflate_buffer.get_ref()[self.last_adler_update_i..self.deflate_buffer.cursor()],
        );
        self.last_adler_update_i = self.deflate_buffer.cursor();
    }

    #[cold]
    fn drain_deflate_buffer(&mut self, dest: &mut OutputWriter<'_>) -> Result<(), DecodingError> {
        self.update_adler();

        let mut drained_bytes = 0;

        for _ in 0..self.scanline_multiples {
            if !self.can_drain_scanline(dest) {
                break;
            }

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
    fn consume_inflated_scanline(
        &mut self,
        start: usize,
        dest: &mut OutputWriter<'_>,
    ) -> Result<usize, DecodingError> {
        let prev_scanline_bytes = self.max_scanline_bytes() + MAX_STRIDE;
        let scanline_bytes = self.scanline_bytes();

        let mut defilterer = InplaceDefilterer::open(
            self.deflate_buffer.get_mut(),
            self.stride,
            start,
            scanline_bytes,
            prev_scanline_bytes,
        );

        defilterer.defilter()?;

        self.output
            .write_scanline(defilterer.filtered_scanline(), dest);
        self.pass.scanline_passed(
            defilterer.both_scanlines(),
            prev_scanline_bytes + 1,
            self.stored_format.bpp(),
            dest,
        );

        Ok(scanline_bytes)
    }

    #[cold]
    fn read_uncompressed_chunk(
        &mut self,
        len: u16,
        dest: &mut OutputWriter<'_>,
    ) -> Result<(), DecodingError> {
        self.compressed_data.unconsume_bitbuf();

        let mut decoded_bytes = 0;

        loop {
            let chunk_len =
                (len - decoded_bytes).min(self.deflate_buffer.remaining().saturating_truncate());

            self.deflate_buffer
                .read_from(&mut **self.compressed_data, chunk_len as usize)?;

            decoded_bytes += chunk_len;

            if decoded_bytes >= len || !self.can_drain_scanline(dest) {
                break;
            }

            self.drain_deflate_buffer(dest)?;
        }

        if len == decoded_bytes {
            // self.compressed_data.align()?;
            self.next_block()?;
        } else {
            self.deflate.set_uncompressed_len(len - decoded_bytes);
            dest.set_full();
        }
        Ok(())
    }

    #[cold]
    fn read_compressed_chunk<const STATIC: bool>(
        &mut self,
        dest: &mut OutputWriter<'_>,
    ) -> Result<(), DecodingError> {
        loop {
            let buffer_full = self.deflate_buffer.cursor() + deflate::MAX_BACKREF_LEN
                >= self.deflate_buffer.capacity();

            if std::hint::unlikely(buffer_full) {
                if self.can_drain_scanline(dest) {
                    self.drain_deflate_buffer(dest)?;
                } else {
                    dest.set_full();
                    break;
                }
            }

            if self
                .deflate
                .decode_compressed::<STATIC>(&mut self.compressed_data, &mut self.deflate_buffer)
            {
                self.next_block()?;
                break;
            }
        }
        Ok(())
    }
}

fn check_header<R: Read>(reader: &mut R) -> Result<(), DecodingError> {
    let mut header = [0u8; 8];
    reader.read_exact(&mut header)?;
    if header != PNG_HEADER {
        return Err(DecodingError::InccorectHeader(header.to_vec()));
    }
    Ok(())
}

fn read_ihdr<R: Read>(parser: &mut ChunkParser<R>) -> Result<Ihdr, DecodingError> {
    let (chunk_header, mut chunk_data) = parser.parse_first_chunk()?;

    if !chunk_header.is_ihdr() {
        return Err(DecodingError::NoIHDR(chunk_header.r#type()));
    }

    let ihdr = Ihdr::read(&mut chunk_data, chunk_header.len())?;

    Ok(ihdr)
}

const fn stored_format(ihdr: &Ihdr) -> PixelFormat {
    PixelFormat::new(
        SampleType::from_bit_depth_unsigned(ihdr.channel_depth).unwrap(),
        match ihdr.color_type {
            ColorType::Truecolor => ChannelLayout::Truecolor,
            ColorType::GrayscaleAlpha => ChannelLayout::GrayscaleAlpha,
            ColorType::TruecolorAlpha => ChannelLayout::TruecolorAlpha,
            ColorType::Grayscale | ColorType::Indexed => ChannelLayout::Grayscale,
        },
    )
}

const fn stride(format: PixelFormat) -> usize {
    let bitspp = format.bpp();
    (bitspp as usize).div_ceil(8)
}
