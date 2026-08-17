use std::io::Read;
use crate::{DecodingError, buffers::CursorVec};

pub trait BitReader {
    fn fill_bitbuf(&mut self);
    fn peek_bits(&mut self, n: u8) -> u64;
    fn peek_bits_nobranch(&mut self, n: u8) -> u64;
    fn consume_bits(&mut self, n: u8);
    fn remaining_bits(&self) -> u8;
    fn read_bits(&mut self, n: u8) -> u64 {
        let bits = self.peek_bits(n);
        self.consume_bits(n);
        bits
    }
    fn read_bits_nobranch(&mut self, n: u8) -> u64 {
        let bits = self.peek_bits_nobranch(n);
        self.consume_bits(n);
        bits
    }
    fn iterate_bits<const BITS: u8>(&mut self) -> BitIterator<'_, Self, BITS> where Self: Sized {
        BitIterator {
            reader: self,
        }
    }
}

pub struct BitIterator<'a, R: BitReader, const BITS: u8> {
    reader: &'a mut R,
}
impl<R: BitReader, const BITS: u8> Iterator for BitIterator<'_, R, BITS> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.reader.read_bits(BITS))
    }
}

pub trait SegmentHeader: Clone {
    const MAX_SIZE: usize;

    fn length(&self) -> usize;
    fn read<R: Read>(reader: &mut R) -> Result<Self, DecodingError> where Self: Sized;
    fn size(&self) -> usize {Self::MAX_SIZE}

    fn is_final(&self) -> bool;
    fn is_special(&self) -> bool {false}
}

pub trait SegmentParser<R: Read> {
    type Header: SegmentHeader;
    type ExtraOut: Default;

    fn context<'s, 'a, 'b, 'c>(&'s mut self) -> (&'a mut CursorVec<u8>, &'b mut R, &'c mut Self::Header)
        where 's: 'a, 's: 'b, 's: 'c;

    fn buffer(&mut self) -> &mut CursorVec<u8> {self.context().0}
    fn header(&mut self) -> &Self::Header {self.context().2}

    fn validate_segment(&mut self) -> Result<(), DecodingError> {
        let (buffer, _, header) = self.context();

        buffer.consume(header.length()); Ok(())
    }

    fn parse_first_chunk(&mut self) -> Result<(Self::Header, &[u8]), DecodingError> {
        self.read_bytes_exact(Self::Header::MAX_SIZE)?;
        self.parse_header()?;
        let prev_header = self.header().clone();

        self.read_to_next_header()?;
        self.parse_header()?;

        self.clear_buffer();

        let chunk_data = &self.buffer().get_ref()[prev_header.size()..prev_header.size() + prev_header.length()];
        Ok((prev_header, chunk_data))
    }

    fn clear_buffer(&mut self) {self.buffer().set_cursor(0);}

    fn handle_special_segment<F>(&mut self, _out: &mut F) -> Result<(), DecodingError>
        where F: FnMut(&Self::Header, &mut CursorVec<u8>, Self::ExtraOut) -> Result<(), DecodingError> {unreachable!()}

    fn parse_chunks<F>(&mut self, mut out: F) -> Result<(), DecodingError>
        where F: FnMut(&Self::Header, &mut CursorVec<u8>, Self::ExtraOut) -> Result<(), DecodingError>
    {
        if self.header().is_special() {self.handle_special_segment(&mut out)?}

        while !self.header().is_final() {
            self.read_to_next_header()?;

            let (buffer, _, header) = self.context();

            let end = buffer.cursor();
            buffer.unconsume(header.length());
            out(header, buffer, Default::default())?;
            buffer.set_cursor(end);

            self.parse_header()?;

            if self.header().is_special() {self.handle_special_segment(&mut out)?}

            self.clear_buffer();
        };

        let len = self.header().length();
        self.read_bytes_exact(len)?;
        self.validate_segment()?;

        Ok(())
    }

    fn read_to_next_header(&mut self) -> Result<(), DecodingError> {
        let len = self.header().length() + Self::Header::MAX_SIZE;

        self.read_bytes_exact(len)?;

        self.validate_segment()
    }

    fn read_bytes(&mut self, len: usize) -> Result<usize, DecodingError> {self.read_bytes_default(len)}

    fn read_bytes_default(&mut self, len: usize) -> Result<usize, DecodingError> {
        let (buffer, reader, _) = self.context();

        if buffer.remaining() <= len {
            buffer.expand(len);
        }

        let cursor = buffer.cursor();
        reader.read(&mut buffer.get_mut()[cursor..cursor + len]).map_err(std::convert::Into::into)
    }

    fn read_bytes_exact(&mut self, len: usize) -> Result<(), DecodingError> {
        match self.read_bytes(len) {
            Ok(n) if n == len => Ok(()),
            Ok(_) => Err(DecodingError::IOError(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "unexpected EOF"))),
            Err(e) => Err(e),
        }
    }

    fn parse_header(&mut self) -> Result<(), DecodingError> {
        let (buffer, _, header) = self.context();

        *header = Self::Header::read(&mut **buffer)?; Ok(())
    }
}
