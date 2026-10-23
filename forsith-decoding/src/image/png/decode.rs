use crate::{
    DecodingError,
    checksums::Adler32,
    decompression::deflate::DeflateDecoder,
    image::{
        PngDecoder,
        outputconverting::OutputWriter,
        png::{
            MAX_STRIDE,
            filtering::{InplaceDefilterer, ScanlinePair},
            interlace::InterlaceState,
            output::PngOutputWriter,
        },
    },
};
use forsith_proc::Deref;
use forsith_shared::{bit::BitReader, buffers::CursorVec};
use std::io::Read;

impl PngDecoder<'_> {
    pub(crate) fn prepare_for_decoding(&mut self) -> Result<(), DecodingError> {
        if self.output.palette().is_none() && self.ihdr.color_type.is_indexed() {
            return Err(DecodingError::NoPallete);
        }

        if self.decode.bufferstate.lz77_size == 0 {
            return Err(DecodingError::NoIDAT);
        }

        self.decode.allocate_buffer(self.max_scanline_bytes());
        self.decode.deflate.next_block(&mut self.compressed_data)
    }

    pub(crate) fn decode_untill_full(
        &mut self,
        dest: &mut OutputWriter<'_>,
    ) -> Result<(), DecodingError> {
        self.decode.decode_until_full(
            &mut self.compressed_data,
            DecodeContext {
                dest,
                interlace: &mut self.interlace,
                output: &self.output,
                stride: self.stride,
            },
        )
    }
}

#[derive(Debug, Default, Deref)]
struct AdlerDecodingState {
    #[deref]
    adler: Adler32,
    last_update: usize,
}

impl AdlerDecodingState {
    pub(crate) fn update(&mut self, data: &CursorVec<u8>) {
        self.adler
            .update(&data.get_ref()[self.last_update..data.cursor()]);
        self.last_update = data.cursor();
    }

    pub(crate) const fn removed(&mut self, bytes: usize) {
        self.last_update -= bytes;
    }

    pub(crate) const fn set_start(&mut self, start: usize) {
        self.last_update = start;
    }

    fn finish(&self, mut compressed_data: BitReader<CursorVec<u8>>) -> Result<(), DecodingError> {
        compressed_data.unconsume_bitbuf();
        self.validate(compressed_data.read_be()?)
    }
}

#[derive(Debug, Default)]
struct BufferState {
    scanlines: usize,
    tail: usize,
    lz77_size: usize,
}

impl BufferState {
    fn consumed(&self, bytes: usize, buf: &mut CursorVec<u8>, prev_scanline: usize) -> usize {
        let range = self.tail + bytes - prev_scanline..buf.cursor();

        let drained_bytes = buf.cursor() - range.len();
        buf.set_cursor(range.len());
        buf.get_mut().copy_within(range, 0);

        drained_bytes
    }

    fn allocate(&mut self, buf: &mut CursorVec<u8>, max_scanline_bytes: usize) {
        self.tail = max_scanline_bytes + MAX_STRIDE;
        buf.get_mut().resize(
            self.tail + self.lz77_size + self.lz77_size.next_multiple_of(max_scanline_bytes),
            0,
        );
        buf.consume(self.tail);
        self.scanlines = self.lz77_size.div_ceil(max_scanline_bytes);
    }
}

#[derive(Debug, Default)]
pub struct DecodingState {
    buffer: CursorVec<u8>,
    bufferstate: BufferState,
    adler: AdlerDecodingState,
    deflate: DeflateDecoder,
}

impl DecodingState {
    fn allocate_buffer(&mut self, max_scanline_bytes: usize) {
        self.bufferstate
            .allocate(&mut self.buffer, max_scanline_bytes);
        self.adler.set_start(self.bufferstate.tail);
    }

    pub(crate) fn open_deflate_stream(
        &mut self,
        reader: &mut impl Read,
    ) -> Result<(), DecodingError> {
        self.bufferstate.lz77_size = self.deflate.open_stream(reader)?;
        Ok(())
    }
}

pub struct DecodeContext<'a, 'dest> {
    dest: &'a mut OutputWriter<'dest>,
    interlace: &'a mut InterlaceState,
    output: &'a PngOutputWriter,
    stride: usize,
}

impl DecodeContext<'_, '_> {
    #[cold]
    fn consume_buffer(
        &mut self,
        bufferstate: &BufferState,
        buffer: &mut CursorVec<u8>,
    ) -> Result<usize, DecodingError> {
        let mut drained_bytes = 0;

        for _ in 0..bufferstate.scanlines {
            if !self.can_drain_scanline() {
                break;
            }

            drained_bytes +=
                self.consume_inflated_scanline(bufferstate.tail + drained_bytes, buffer)?;
        }

        Ok(drained_bytes)
    }

    #[inline(always)]
    fn consume_inflated_scanline(
        &mut self,
        start: usize,
        buffer: &mut CursorVec<u8>,
    ) -> Result<usize, DecodingError> {
        let scanline_bytes = self.interlace.scanline_bytes();

        let mut scanlines = InplaceDefilterer::defilter_scanline(
            ScanlinePair::open(
                buffer.get_mut(),
                start,
                self.interlace.scanline_bytes(),
                self.interlace.max_scanline_bytes() + MAX_STRIDE,
            ),
            self.stride,
        )?;

        self.output
            .write_scanline(scanlines.filtered_scanline_pixels(), self.dest);
        self.interlace.scanline_passed(scanlines, self.dest);

        Ok(scanline_bytes)
    }

    fn can_drain_scanline(&self) -> bool {
        self.output
            .can_push(self.dest, self.interlace.scanline_pixel_bytes())
            || self.interlace.interlacing_enabled()
    }

    fn drain_buffer(
        &mut self,
        buffer: &mut CursorVec<u8>,
        bufferstate: &BufferState,
        adler: &mut AdlerDecodingState,
    ) -> Result<bool, DecodingError> {
        if !self.can_drain_scanline() {
            return Ok(false);
        }

        adler.update(buffer);

        let consumed_bytes = self.consume_buffer(bufferstate, buffer)?;

        adler.removed(bufferstate.consumed(
            consumed_bytes,
            buffer,
            self.interlace.max_scanline_bytes() + MAX_STRIDE,
        ));

        Ok(true)
    }
}

impl DecodingState {
    pub(crate) fn decode_until_full(
        &mut self,
        compressed_data: &mut BitReader<CursorVec<u8>>,
        mut context: DecodeContext<'_, '_>,
    ) -> Result<(), DecodingError> {
        self.deflate
            .decode_untill_full(compressed_data, &mut self.buffer, |buffer| {
                context.drain_buffer(buffer, &self.bufferstate, &mut self.adler)
            })?;

        if self.deflate.is_finished() {
            self.adler.update(&self.buffer);
            self.consume_whole_buffer(context)?;

            if self.bufferstate.tail == self.buffer.cursor() && compressed_data.capacity() > 0 {
                self.adler.finish(core::mem::take(compressed_data))?;
            }
        }

        Ok(())
    }

    fn consume_whole_buffer(
        &mut self,
        mut context: DecodeContext<'_, '_>,
    ) -> Result<(), DecodingError> {
        while context.can_drain_scanline()
            && self.buffer.cursor() - self.bufferstate.tail >= context.interlace.scanline_bytes()
        {
            let consumed_bytes =
                context.consume_inflated_scanline(self.bufferstate.tail, &mut self.buffer)?;

            self.bufferstate.tail += consumed_bytes;
        }

        Ok(())
    }
}
