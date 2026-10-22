use crate::{
    DecodingError,
    checksums::Adler32,
    decompression::deflate::{self, BlockType, DeflateDecoder},
    image::{
        PngDecoder,
        outputconverting::OutputWriter,
        png::{
            MAX_STRIDE,
            filtering::{InplaceDefilterer, ScanlinePair},
        },
    },
};
use forsith_shared::buffers::CursorVec;
use std::io::Read;

#[derive(Debug, Default)]
pub struct DecodingState {
    deflate_buffer: CursorVec<u8>,
    scanline_multiples: usize,
    deflate_buffer_tail: usize,
    last_adler_update_i: usize,
    adler: Adler32,
    deflate: DeflateDecoder,
}

impl DecodingState {
    pub(crate) fn update_adler(&mut self) {
        self.adler.update(
            &self.deflate_buffer.get_ref()[self.last_adler_update_i..self.deflate_buffer.cursor()],
        );
        self.last_adler_update_i = self.deflate_buffer.cursor();
    }

    pub(crate) fn deflate_buffer_remove(&mut self, bytes: usize, prev_scanline: usize) {
        let range = self.deflate_buffer_tail + bytes - prev_scanline..self.deflate_buffer.cursor();

        self.last_adler_update_i -= self.deflate_buffer.cursor() - range.len();

        self.deflate_buffer.set_cursor(range.len());
        self.deflate_buffer.get_mut().copy_within(range, 0);
    }

    pub(crate) fn open_deflate_stream(
        &mut self,
        reader: &mut impl Read,
        max_scanline_bytes: usize,
    ) -> Result<(), DecodingError> {
        let lz77_buffer_size = self.deflate.open_stream(reader)?;

        let scanline_padding = max_scanline_bytes + MAX_STRIDE;
        self.deflate_buffer = CursorVec::new(
            scanline_padding
                + lz77_buffer_size
                + lz77_buffer_size.next_multiple_of(max_scanline_bytes),
        );

        self.deflate_buffer.consume(scanline_padding);
        self.deflate_buffer_tail = scanline_padding;
        self.last_adler_update_i = scanline_padding;

        self.scanline_multiples = (self.deflate_buffer.capacity() - lz77_buffer_size - MAX_STRIDE)
            / max_scanline_bytes
            - 1;

        Ok(())
    }
}

impl PngDecoder<'_> {
    #[cold]
    pub(crate) fn drain_deflate_buffer(
        &mut self,
        dest: &mut OutputWriter<'_>,
    ) -> Result<(), DecodingError> {
        self.decode.update_adler();

        let mut drained_bytes = 0;

        for _ in 0..self.decode.scanline_multiples {
            if !self.can_drain_scanline(dest) {
                break;
            }

            drained_bytes +=
                self.consume_inflated_scanline(self.decode.deflate_buffer_tail, dest)?;
        }

        self.deflate_buffer_remove(drained_bytes);

        Ok(())
    }

    fn deflate_buffer_remove(&mut self, bytes: usize) {
        let prev_scanline = self.max_scanline_bytes() + MAX_STRIDE;
        self.decode.deflate_buffer_remove(bytes, prev_scanline);
    }

    #[inline(always)]
    fn consume_inflated_scanline(
        &mut self,
        start: usize,
        dest: &mut OutputWriter<'_>,
    ) -> Result<usize, DecodingError> {
        let prev_scanline_bytes = self.max_scanline_bytes() + MAX_STRIDE;
        let scanline_bytes = self.scanline_bytes();

        let scanline_pair = ScanlinePair::open(
            self.decode.deflate_buffer.get_mut(),
            start,
            scanline_bytes,
            prev_scanline_bytes,
        );

        let mut defilterer = InplaceDefilterer::open(scanline_pair, self.stride);
        defilterer.defilter()?;
        let mut scanline_pair = defilterer.decompose();

        self.output
            .write_scanline(scanline_pair.filtered_scanline_pixels(), dest);

        self.interlace
            .scanline_passed(scanline_pair, self.stored_format.bpp(), dest);

        Ok(scanline_bytes)
    }

    pub(crate) fn prepare_for_decoding(&mut self) -> Result<(), DecodingError> {
        if self.output.palette().is_none() && self.ihdr.color_type.is_indexed() {
            return Err(DecodingError::NoPallete);
        }

        if self.decode.deflate_buffer.get_ref().is_empty() {
            return Err(DecodingError::NoIDAT);
        }

        self.next_block()
    }

    #[cold]
    fn next_block(&mut self) -> Result<(), DecodingError> {
        if self.decode.deflate.next_block(&mut self.compressed_data)? {
            self.finish_decoding()?;
        }

        Ok(())
    }

    #[cold]
    fn finish_decoding(&mut self) -> Result<(), DecodingError> {
        self.decode.update_adler();

        self.compressed_data.unconsume_bitbuf();
        self.decode
            .adler
            .validate(self.compressed_data.read_be()?)?;

        Ok(())
    }

    const fn can_drain_scanline(&self, dest: &OutputWriter<'_>) -> bool {
        dest.remaining() * 8 / self.output_format().bpp() as usize
            * self.stored_format().bpp() as usize
            >= self.scanline_pixel_bytes() * 8
            || self.interlace.interlacing_enabled()
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
            let chunk_len = (len - decoded_bytes)
                .min(self.decode.deflate_buffer.remaining().saturating_truncate());

            self.decode
                .deflate_buffer
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
            self.decode
                .deflate
                .set_uncompressed_len(len - decoded_bytes);
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
            let buffer_full = self.decode.deflate_buffer.cursor() + deflate::MAX_BACKREF_LEN
                >= self.decode.deflate_buffer.capacity();

            if std::hint::unlikely(buffer_full) {
                if self.can_drain_scanline(dest) {
                    self.drain_deflate_buffer(dest)?;
                } else {
                    dest.set_full();
                    break;
                }
            }

            if self.decode.deflate.decode_compressed::<STATIC>(
                &mut self.compressed_data,
                &mut self.decode.deflate_buffer,
            ) {
                self.next_block()?;
                break;
            }
        }
        Ok(())
    }

    pub(crate) fn decode_chunk(
        &mut self,
        dest: &mut OutputWriter<'_>,
    ) -> Result<bool, DecodingError> {
        match self.decode.deflate.cur_block().ty {
            BlockType::Uncompressed(len) => {
                self.read_uncompressed_chunk(len, dest)?;
            }
            BlockType::CompressedFixed => {
                self.read_compressed_chunk::<true>(dest)?;
            }
            BlockType::CompressedDynamic => {
                self.read_compressed_chunk::<false>(dest)?;
            }
            BlockType::Finished => {
                while self.can_drain_scanline(dest)
                    && self.decode.deflate_buffer.cursor() - self.decode.deflate_buffer_tail
                        >= self.scanline_bytes()
                {
                    let consumed_bytes =
                        self.consume_inflated_scanline(self.decode.deflate_buffer_tail, dest)?;

                    self.decode.deflate_buffer_tail += consumed_bytes;
                }

                return Ok(true);
            }
        }

        Ok(false)
    }
}
