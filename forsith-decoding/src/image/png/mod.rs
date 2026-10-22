use crate::{
    DecodingError,
    bit::BitReader,
    buffers::CursorVec,
    checksums,
    decompression::deflate::{self},
    image::{
        ChannelLayout, ImageDecoder, PixelFormat, SampleType,
        outputconverting::OutputWriter,
        png::{
            chunks::{ColorType, Ihdr},
            decode::DecodingState,
            deflate::MAX_BACKREF_LEN,
            interlace::InterlaceState,
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

mod interlace;

mod decode;

const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const MAX_STRIDE: usize = 8;

pub struct PngDecoder<'a> {
    compressed_data: BitReader<CursorVec<u8>>,
    ihdr: Ihdr,
    stride: usize,
    stored_format: PixelFormat,
    interlace: InterlaceState,
    output: PngOutputWriter,
    decode: DecodingState,
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
            decode: DecodingState::default(),
            output: PngOutputWriter::new(ihdr.color_type, stored_format),
            interlace: InterlaceState::new(&ihdr, stored_format),
            stride: stride(stored_format),
            stored_format,
            ihdr,
            phantom: std::marker::PhantomData,
        };

        decoder.parse_everything(chunk_parser)?;
        decoder.prepare_for_decoding()?;

        Ok(decoder)
    }

    #[cold]
    fn read(&mut self, dest: &mut [u8]) -> Result<usize, DecodingError> {
        if dest.len() < self.min_buf_size() {
            return Err(DecodingError::TinyDestBuf(dest.len()));
        }

        let mut dest = OutputWriter::new(dest, self.output_format());
        self.interlace.update_dest(&mut dest);

        while !dest.is_full() && !self.decode_chunk(&mut dest)? {}

        if self.interlace.interlacing_enabled() && !dest.is_empty() {
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
    const fn stored_format(&self) -> PixelFormat {
        self.stored_format
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
