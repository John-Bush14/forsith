use std::ops::{Index, Range};

use crate::{DecodingError, Num, png::readers::BitReader};

const MAX_COLEN: u8 = 18;
const CODE_LENGTH_ORDER: [u8; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
const MAX_SINGLE_TABLE_COLEN: u8 = 9; // maximum code length for a single table (fixed or dynamic)

// (base, extra_bits) for length symbols 257..=285
const LENGTH_TABLE: [(u16, u8); 29] = [
    (3,0),(4,0),(5,0),(6,0),(7,0),(8,0),(9,0),(10,0),
    (11,1),(13,1),(15,1),(17,1),
    (19,2),(23,2),(27,2),(31,2),
    (35,3),(43,3),(51,3),(59,3),
    (67,4),(83,4),(99,4),(115,4),
    (131,5),(163,5),(195,5),(227,5),
    (258,0),
];

// (base, extra_bits) for distance codes 0..=29
const DISTANCE_TABLE: [(u16, u8); 30] = [
    (1,0),(2,0),(3,0),(4,0),
    (5,1),(7,1),
    (9,2),(13,2),
    (17,3),(25,3),
    (33,4),(49,4),
    (65,5),(97,5),
    (129,6),(193,6),
    (257,7),(385,7),
    (513,8),(769,8),
    (1025,9),(1537,9),
    (2049,10),(3073,10),
    (4097,11),(6145,11),
    (8193,12),(12289,12),
    (16385,13),(24577,13),
];

pub fn decode_length<R: BitReader>(symbol: u16, reader: &mut R) -> std::io::Result<u16> {
    let (base, extra) = LENGTH_TABLE[(symbol - 257) as usize];
    Ok(base + reader.read_bits(extra)? as u16)
}

pub fn decode_distance<R: BitReader>(code: u16, reader: &mut R) -> std::io::Result<u16> {
    let (base, extra) = DISTANCE_TABLE[code as usize];
    Ok(base + reader.read_bits(extra)? as u16)
}

#[derive(Debug, Default)]
pub struct HuffmanTree<S: Num> {
    table: Vec<Entry<S>>, // (symbol, code length)
    max_colen: u8,
    // subtables: Vec<S>,
}
#[derive(Debug, Clone, Copy)]
enum Entry<S: Num> {
    Empty,
    Symbol(S, u8), // (symbol, code length)
    // Sub
}
impl<S: Num> HuffmanTree<S> {
    pub fn load(&mut self, code_lengths: &[u8]) -> Result<(), DecodingError> {
        self.max_colen = code_lengths.iter().copied().max().unwrap_or(0);
        if self.max_colen > MAX_COLEN {
            return Err(DecodingError::InvalidCodeLength(self.max_colen));
        }

        let mut colen_counts = Self::get_colen_counts(code_lengths)?;

        if colen_counts[2] == 0 && colen_counts[1] == 1 {
            colen_counts[0] = 1; // make single symbol have code 1
        }

        let first_codes = self.generate_first_codes(&colen_counts);

        self.generate_table(code_lengths, first_codes)?;

        Ok(())
    }

    fn generate_table(&mut self, code_lengths: &[u8], mut next_code: [u32; MAX_COLEN as usize + 1]) -> Result<(), DecodingError> {
        Vec::resize(&mut self.table, 1 << self.max_colen, Entry::Empty);

        for (symbol, &colen) in code_lengths.iter().enumerate().map(|(s, l)| (S::try_from(s), l)) {
            if colen == 0 {continue;}
            let Ok(symbol) = symbol else {return Err(DecodingError::InvalidSymbol(std::mem::size_of::<S>()))};

            let code = next_code[colen as usize];
            next_code[colen as usize] += 1;

            let code = reverse_bits(code, colen as _);

            let filler = 1 << (self.max_colen - colen);

            for i in 0..filler {
                self.table[(code as usize) | (i << colen)] = Entry::Symbol(symbol, colen);
            }
        }

        Ok(())
    }

    fn get_colen_counts(colens: &[u8]) -> Result<[u16; MAX_COLEN as usize + 1], DecodingError> {
        let mut colen_count = [0u16; MAX_COLEN as usize + 1];

        for &colen in colens {
            if colen > MAX_COLEN {
                return Err(DecodingError::InvalidCodeLength(colen));
            }

            colen_count[colen as usize] += 1;
        }
        colen_count[0] = 0;

        Ok(colen_count)
    }

    fn generate_first_codes(&self, colen_counts: &[u16; MAX_COLEN as usize + 1]) -> [u32; MAX_COLEN as usize + 1] {
        let mut first_codes = [0u32; MAX_COLEN as usize + 1];

        for i in 1..=self.max_colen as usize {
            first_codes[i] = (first_codes[i - 1] + colen_counts[i - 1] as u32) << 1;
        }

        first_codes
    }

    pub fn decode_symbol<R: BitReader>(&self, reader: &mut R) -> Result<S, DecodingError> {
        let code = reader.peek_bits(self.max_colen)?;

        let (symbol, colen) = match self.table[code] {
            Entry::Symbol(s, c) => (s, c),
            Entry::Empty => return Err(DecodingError::UndefinedHuffmanCode(code as u32)),
        };

        reader.consume_bits(colen);

        Ok(symbol)
    }

    pub fn iter_decode<'a, R: BitReader>(&'a self, reader: &'a mut R) -> HuffmanDecoder<'a, S, R> {
        HuffmanDecoder {
            tree: self,
            reader,
        }
    }
}

pub struct HuffmanDecoder<'a, S: Num, R: BitReader> {
    tree: &'a HuffmanTree<S>,
    reader: &'a mut R,
}
impl<'a, S: Num, R: BitReader> Iterator for HuffmanDecoder<'a, S, R> {
    type Item = Result<S, DecodingError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.tree.decode_symbol(self.reader))
    }
}

fn reverse_bits(value: u32, bits: usize) -> u32 {
    value.reverse_bits() >> (32 - bits)
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum BlockType {
    Uncompressed(u16),
    CompressedFixed,
    CompressedDynamic,
    #[default]
    Finished,
}
#[derive(Debug, Default)]
pub struct Block {
    pub last: bool,
    pub r#type: BlockType,
    pub litlen_tree: HuffmanTree<u16>,
    pub distance_tree: HuffmanTree<u16>,
    pub codlen_tree: HuffmanTree<u8>,
}
impl Block {
    pub fn load_block<R: BitReader>(&mut self, reader: &mut R) -> Result<(), DecodingError> {
        self.last = reader.read_bits(1)? == 1;
        self.load_compression_type(reader)?;

        if self.r#type == BlockType::CompressedDynamic {
            self.load_trees(reader)?;
        }

        Ok(())
    }

    fn load_trees<R: BitReader>(&mut self, reader: &mut R) -> Result<(), DecodingError> {
        let hlit: u16 = reader.read_bits(5)? as u16 + 257;
        let hdist: u16 = reader.read_bits(5)? as u16 + 1;
        let hclen: u16 = reader.read_bits(4)? as u16 + 4;

        let mut codlen_codelengths = vec![0u8; 19];
        for (i, colen) in reader.iterate_bits(3).take(hclen as usize).enumerate() {
            let index = CODE_LENGTH_ORDER[i] as usize;
            codlen_codelengths[index] = colen? as u8;
        }
        self.codlen_tree.load(&codlen_codelengths)?;

        // Decode hlit + hdist code lengths, expanding RLE symbols 16/17/18
        let total = hlit as usize + hdist as usize;
        let mut all_codelengths = Vec::with_capacity(total);
        while all_codelengths.len() < total {
            let symbol = self.codlen_tree.decode_symbol(reader)?;
            match symbol {
                0..=15 => all_codelengths.push(symbol as u8),
                16 => {
                    let repeat = reader.read_bits(2)? as u8 + 3;
                    let prev = *all_codelengths.last().unwrap_or(&0);
                    for _ in 0..repeat { all_codelengths.push(prev); }
                }
                17 => {
                    let repeat = reader.read_bits(3)? as u8 + 3;
                    for _ in 0..repeat { all_codelengths.push(0); }
                }
                18 => {
                    let repeat = reader.read_bits(7)? as u8 + 11;
                    for _ in 0..repeat { all_codelengths.push(0); }
                }
                _ => unreachable!(),
            }
        }

        self.litlen_tree.load(&all_codelengths[..hlit as usize])?;
        self.distance_tree.load(&all_codelengths[hlit as usize..])?;

        Ok(())
    }

    fn load_compression_type<R: BitReader>(&mut self, reader: &mut R) -> Result<(), DecodingError> {
        match reader.read_bits(2)? {
            0 => {
                let len = reader.read_bits(16)? as u16;
                let nlen = reader.read_bits(16)? as u16;

                if len != !nlen {
                    return Err(DecodingError::BlockLengthMismatch(len, nlen));
                }

                self.r#type = BlockType::Uncompressed(len)
            },
            1 => self.r#type = BlockType::CompressedFixed,
            2 => self.r#type = BlockType::CompressedDynamic,
            _ => return Err(DecodingError::ReservedCompressionMethod),
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct LZ77Buffer {
    pub buffer: Vec<u8>,
    pub lz77_buffer_size: usize,
    pub index: usize,
}
impl LZ77Buffer {
    pub fn new(size: usize, lz77_buffer_size: usize) -> Self {
        Self {
            buffer: vec![0; size],
            index: 0,
            lz77_buffer_size
        }
    }

    pub fn push_literal(&mut self, b: u8) {
        unsafe {
            *self.buffer.get_unchecked_mut(self.index) = b;
        }
        self.index += 1;
    }

    pub fn backreference(&mut self, distance: usize) -> u8 {
        let start = self.index - distance;
        unsafe {*self.buffer.get_unchecked(start)}
    }

    pub fn slice(&self, range: Range<usize>) -> &[u8] {
        unsafe {&self.buffer.get_unchecked(range)}
    }

    pub fn shift(&mut self, new_start: usize) {
        self.buffer.copy_within(new_start..self.index, 0);
        self.index -= new_start;
    }

    pub fn capacity(&self) -> usize {self.buffer.len()}
    pub fn len(&self) -> usize {self.index}
    pub fn remaining(&self) -> usize {self.capacity() - self.len()}
}

impl Index<usize> for LZ77Buffer {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe {self.buffer.get_unchecked(index)}
    }
}
