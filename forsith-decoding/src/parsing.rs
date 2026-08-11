use std::io::{Read, Seek};
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
    const SIZE: usize;

    fn length(&self) -> usize;
    fn read<R: Read>(reader: &mut R) -> Result<Self, DecodingError> where Self: Sized;
}

pub trait SegmentParser<R: Read> {
    type Header: SegmentHeader;

    fn context<'s, 'a, 'b, 'c>(&'s mut self) -> (&'a mut CursorVec<u8>, &'b mut R, &'c mut Self::Header)
        where 's: 'a, 's: 'b, 's: 'c;

    fn buffer(&mut self) -> &mut CursorVec<u8> {self.context().0}
    fn header(&mut self) -> &Self::Header {self.context().2}

    fn validate_segment(&mut self) -> Result<(), DecodingError> {
        let (buffer, _, header) = self.context();

        buffer.seek_relative(header.length() as i64).map_err(|e| e.into())
    }

    fn parse_first_chunk(&mut self) -> Result<(Self::Header, &[u8]), DecodingError> {
        let (buffer, reader, _) = self.context();

        buffer.fill_from(reader, Self::Header::SIZE)?;
        self.parse_header()?;
        let prev_header = self.header().clone();

        self.read_segment_and_next_header()?;
        self.validate_segment()?;

        self.parse_header()?;

        self.buffer().set_cursor(0);

        let chunk_data = &self.buffer().get_ref()[Self::Header::SIZE..Self::Header::SIZE + prev_header.length()];
        Ok((prev_header, chunk_data))
    }

    fn parse_chunks<F>(&mut self, out: F) -> Result<(), DecodingError>
        where F: FnMut(&Self::Header, &mut CursorVec<u8>) -> Result<(), DecodingError>;

    fn read_segment_and_next_header(&mut self) -> Result<(), DecodingError> {
        let (buffer, reader, header) = self.context();
        let len = header.length() + Self::Header::SIZE;

        if buffer.remaining() <= len {
            buffer.expand(len);
        }

        buffer.fill_from(reader, len).map_err(|e| e.into())
    }

    fn parse_header(&mut self) -> Result<(), DecodingError> {
        let (buffer, _, header) = self.context();

        *header = Self::Header::read(&mut **buffer)?; Ok(())
    }
}
