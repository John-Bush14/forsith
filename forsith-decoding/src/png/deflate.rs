use core::panic;

use num_enum::FromPrimitive;

use crate::{DecodingError, Num, png::readers::BitReader};

const MAX_COLEN: u8 = 18;
const CODE_LENGTH_ORDER: [u8; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

#[derive(Debug, Default)]
pub struct HuffmanTree<S: Num> {
    table: Vec<Entry<S>>, // (symbol, code length)
    max_colen: u8,
    //nested_tree: Option<Box<HuffmanTree<S>>>,
}
#[derive(Debug, Clone, Copy)]
enum Entry<S: Num> {
    Empty,
    Symbol(S, u8), // (symbol, code length)
    //NestedTree
}
impl<S: Num> HuffmanTree<S> {
    pub fn load(&mut self, code_lengths: &[u8]) -> Result<(), DecodingError> {
        let mut colen_count = [0u8; MAX_COLEN as usize + 1];
        self.max_colen = 0;

        for &colen in code_lengths {
            if colen > MAX_COLEN {
                return Err(DecodingError::InvalidCodeLength(colen));
            }

            colen_count[colen as usize] += 1;
            self.max_colen = self.max_colen.max(colen);
        };
        colen_count[0] = 0;

        if colen_count[2] == 0 && colen_count[1] == 1 {
            colen_count[0] = 1; // make single symbol have code 1
        }

        let mut first_codes = [0u32; MAX_COLEN as usize + 1];
        for i in 1..=self.max_colen as usize {
            first_codes[i] = (first_codes[i - 1] + colen_count[i - 1] as u32) << 1;
        }

        Vec::resize(&mut self.table, 1 << self.max_colen, Entry::Empty);

        for (symbol, &colen) in code_lengths.iter().enumerate().map(|(s, l)| (S::try_from(s), l)) {
            if colen == 0 {continue;}
            let Ok(symbol) = symbol else {return Err(DecodingError::InvalidSymbol(std::mem::size_of::<S>()))};

            let code = first_codes[colen as usize];
            first_codes[colen as usize] += 1;

            let code = reverse_bits(code, colen as _);

            let filler = 1 << (self.max_colen - colen);

            for i in 0..filler {
                self.table[(code as usize) | (i << colen)] = Entry::Symbol(symbol, colen);
            }
        }

        Ok(())
    }

    fn decode_symbol<R: BitReader>(&self, reader: &mut R) -> Result<S, DecodingError> {
        let code = reader.peek_bits(self.max_colen)?;

        let (symbol, colen) = match self.table[code] {
            Entry::Symbol(s, c) => (s, c),
            Entry::Empty => return Err(DecodingError::UndefinedHuffmanCode(code as u32)),
        };

        reader.consume_bits(colen);

        Ok(symbol)
    }

    fn iter_decode<'a, R: BitReader>(&'a self, reader: &'a mut R) -> HuffmanDecoder<'a, S, R> {
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

fn reverse_bits(mut value: u32, bits: usize) -> u32 {
    let mut result = 0;

    for _ in 0..bits {
        result <<= 1;
        result |= value & 1;
        value >>= 1;
    }

    result
}

#[derive(Debug, Default, PartialEq, Eq)]
enum CompressionType {
    Uncompressed(u16),
    CompressedFixed,
    CompressedDynamic,
    #[default]
    Reserved,
}
#[derive(Debug, Default)]
pub struct Block {
    pub last: bool,
    pub compression_type: CompressionType,
    pub litlen_tree: HuffmanTree<u16>,
    pub distance_tree: HuffmanTree<u16>,
    pub codlen_tree: HuffmanTree<u8>,
}
impl Block {
    pub fn load_block<R: BitReader>(&mut self, reader: &mut R) -> Result<(), DecodingError> {
        self.last = reader.read_bits(1)? == 1;
        self.load_compression_type(reader)?;

        if self.compression_type == CompressionType::CompressedDynamic {
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

                self.compression_type = CompressionType::Uncompressed(len)
            },
            1 => self.compression_type = CompressionType::CompressedFixed,
            2 => self.compression_type = CompressionType::CompressedDynamic,
            _ => self.compression_type = CompressionType::Reserved,
        }

        Ok(())
    }
}
