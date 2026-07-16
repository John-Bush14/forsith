use const_for::const_for;

use crate::{DecodingError, png::reader::BitReader};

const MAX_COLEN: u8 = 17;
const CODE_LENGTH_ORDER: [u8; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
const MAX_ROOT_TABLE_COLEN: u8 = 9;

const MAX_LITLEN_SUBTABLE_ENTIES: usize = 340;
const MAX_DISTANCE_SUBTABLE_ENTRIES: usize = 80;

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

pub const STATIC_LITLEN_TREE: LitLenTree = {
    let mut tree = HuffmanTree::default();

    let mut lengths = [0u8; 288];

    const_for!(i in 0..144 => {lengths[i] = 8;});
    const_for!(i in 144..256 => {lengths[i] = 9;});
    const_for!(i in 256..280 => {lengths[i] = 7;});
    const_for!(i in 280..288 => {lengths[i] = 8;});

    if let Err(_e) = tree.load(&lengths) { panic!("loading static litlen tree failed") };

    tree
};
pub const STATIC_DISTANCE_TREE: DistanceTree = {
    let mut tree = HuffmanTree::default();

    let lengths = [5u8; 30];

    if let Err(_e) = tree.load(&lengths) { panic!("loading static distance tree failed") };

    tree
};

type LitLenTree = HuffmanTree<MAX_ROOT_TABLE_COLEN, MAX_LITLEN_SUBTABLE_ENTIES>;
type DistanceTree = HuffmanTree<MAX_ROOT_TABLE_COLEN, MAX_DISTANCE_SUBTABLE_ENTRIES>;
type CodlenTree = HuffmanTree<7, 0>;


pub fn decode_length<R: BitReader>(symbol: u16, reader: &mut R) -> std::io::Result<u16> {
    let (base, extra) = LENGTH_TABLE[(symbol - 257) as usize];
    Ok(base + reader.read_bits(extra)? as u16)
}

pub fn decode_distance<R: BitReader>(code: u16, reader: &mut R) -> std::io::Result<u16> {
    let (base, extra) = DISTANCE_TABLE[code as usize];
    Ok(base + reader.read_bits(extra)? as u16)
}

#[derive(Debug)]
#[repr(C)]
pub struct HuffmanTree<const MAX_ROOT_BITS: u8, const MAX_SUBTABLE_ENTRIES: usize>
where [(); (1 << MAX_ROOT_BITS as usize) + MAX_SUBTABLE_ENTRIES]:
{
    table: [Entry; (1 << MAX_ROOT_BITS as usize) + MAX_SUBTABLE_ENTRIES],
    root_bits: u8,
    sub_bits: u8,
    next_subtable: usize,
    generation: usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Entry(u32);

impl Entry {
    const fn empty() -> Self {Entry(0)}

    const fn new_symbol(symbol: u16, colen: u8) -> Self {
        Self(symbol as u32 | ((colen as u32) << 24) | (0b01 << 16) as u32)
    }

    /// offset from root table end
    const fn new_subtable(index: usize, bits: u8) -> Self {
        Self(index as u32 | ((bits as u32) << 24) | (0b10 << 16) as u32)
    }

    /// not to be used in decoding as can't be differentiated
    const fn new_longcode(symbol: u16, code: u16) -> Self {
        Self(symbol as u32 | (code as u32) << 16)
    }

    const fn symbol(&self) -> u16 {
        unsafe {(self.0 & u16::MAX as u32).unchecked_cast()}
    }
    const fn subtable_index(&self) -> usize {self.symbol() as usize}

    const fn colen(&self) -> u8 {
        unsafe {(self.0 >> 24).unchecked_cast()}
    } const fn subtable_bits(&self) -> u8 {self.colen()}

    const fn code(&self) -> u16 {
        unsafe {(self.0 >> 16).unchecked_cast()}
    }

    const fn _is_symbol(&self) -> bool {self.0 >> 16 & 0b11 == 1}
    const fn is_subtable(&self) -> bool {self.0 >> 16 & 0b11 == 2}

    const fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl<const MAX_ROOT_BITS: u8, const MAX_SUBTABLE_ENTRIES: usize> HuffmanTree<MAX_ROOT_BITS, MAX_SUBTABLE_ENTRIES>
where [(); (1 << MAX_ROOT_BITS as usize) + MAX_SUBTABLE_ENTRIES]:
{
    const fn default() -> Self {
        Self {
            table: [Entry::empty(); (1 << MAX_ROOT_BITS as usize) + MAX_SUBTABLE_ENTRIES],
            root_bits: 0,
            sub_bits: 0,
            next_subtable: 0,
            generation: 0,
        }
    }

    pub const fn load(&mut self, code_lengths: &[u8]) -> Result<(), DecodingError> {
        let (mut colen_counts, max_colen) = Self::get_colen_counts(code_lengths);

        self.root_bits = max_colen.min(MAX_ROOT_BITS);

        if MAX_SUBTABLE_ENTRIES > 0 {
            self.sub_bits = max_colen.saturating_sub(MAX_ROOT_BITS);
            self.generation = (self.generation + 1) & (1 << MAX_ROOT_BITS);
        }

        if colen_counts[2] == 0 && colen_counts[1] == 1 {
            colen_counts[0] = 1; // make single symbol have code 1
        }

        let first_codes = self.generate_first_codes(&colen_counts);

        self.generate_table(code_lengths, first_codes)?;

        Ok(())
    }

    const fn generate_table(&mut self, code_lengths: &[u8], mut next_code: [u32; MAX_COLEN as usize + 1]) -> Result<(), DecodingError> {
        let mut longcodes = [Entry::empty(); MAX_SUBTABLE_ENTRIES];
        self.next_subtable = 0;

        let mut i = 0usize;
        while i < code_lengths.len() {
            let colen = code_lengths[i];
            let symbol = i as u16; i += 1;

            if colen == 0 {continue;}

            let code = next_code[colen as usize];
            next_code[colen as usize] += 1;

            let code = reverse_bits(code, colen as _);

            if MAX_SUBTABLE_ENTRIES == 0 || colen <= MAX_ROOT_TABLE_COLEN {
                let filler = 1 << (self.root_bits - colen);

                let mut i = 0;
                while i < filler {
                    self.table[(code as usize) | (i << colen)]  = Entry::new_symbol(symbol, colen);

                    i += 1;
                }

                continue;
            }

            let subcolen = colen - MAX_ROOT_TABLE_COLEN;
            let root = (code & ((1 << MAX_ROOT_BITS) - 1)) as usize;

            if self.table[root].subtable_index() != self.generation {
                self.table[root] = Entry::new_subtable(self.generation, subcolen);
            } else {
                self.table[root] = Entry::new_subtable(self.generation, self.table[root].subtable_bits().max(subcolen));
            }

            longcodes[self.next_subtable] = Entry::new_longcode(symbol, code as u16);
            self.next_subtable += 1;
        }

        if MAX_SUBTABLE_ENTRIES == 0 {return Ok(());}

        let longcodes_len = self.next_subtable;
        self.next_subtable = 1 << MAX_ROOT_BITS;

        let mut i = 0;
        while i < longcodes_len {
            let entry = longcodes[i]; i += 1;

            let (symbol, code) = (entry.symbol(), entry.code());
            let colen = code_lengths[symbol as usize];

            let root = code & ((1 << MAX_ROOT_BITS) - 1);
            let subcode = code >> MAX_ROOT_BITS;

            let root_entry = &mut self.table[root as usize];

            if root_entry.subtable_index() == 0 {
                *root_entry = Entry::new_subtable(self.next_subtable, root_entry.subtable_bits());
                self.next_subtable += 1 << root_entry.subtable_bits();
            }

            let subcolen = colen - MAX_ROOT_BITS;

            let subtable_start = root_entry.subtable_index();
            let subtable_bits = root_entry.subtable_bits();

            let filler = 1 << (subtable_bits - subcolen);

            let mut i = 0;
            while i < filler {
                self.table[subtable_start + ((subcode as usize) | (i << subcolen))] = Entry::new_symbol(symbol, colen);

                i += 1;
            }
        }

        Ok(())
    }

    const fn get_colen_counts(colens: &[u8]) -> ([u16; MAX_COLEN as usize + 1], u8) {
        let mut colen_count = [0u16; MAX_COLEN as usize + 1];
        let mut max_colen = 0;

        let mut i = 0;
        while i < colens.len() {
            let colen = colens[i]; i += 1;

            colen_count[colen as usize] += 1;

            if colen > max_colen {
                max_colen = colen;
            }
        }
        colen_count[0] = 0;

        (colen_count, max_colen)
    }

    const fn generate_first_codes(&self, colen_counts: &[u16; MAX_COLEN as usize + 1]) -> [u32; MAX_COLEN as usize + 1] {
        let mut first_codes = [0u32; MAX_COLEN as usize + 1];

        let mut i = 0;
        while i < (self.root_bits + self.sub_bits) as usize {
            i += 1;

            first_codes[i] = (first_codes[i - 1] + colen_counts[i - 1] as u32) << 1;
        }

        first_codes
    }

    pub fn decode_symbol<R: BitReader>(&self, reader: &mut R) -> Result<u16, DecodingError> {
        let code = reader.peek_bits(self.root_bits)?;

        let mut entry = self.table[code];

        if MAX_SUBTABLE_ENTRIES != 0 && entry.is_subtable() {
            let subtable_bits = reader.peek_bits(entry.subtable_bits() + MAX_ROOT_BITS)? >> MAX_ROOT_BITS;
            entry = self.table[entry.subtable_index() + subtable_bits]
        }

        reader.consume_bits(entry.colen());
        Ok(entry.symbol())
    }
}

const fn reverse_bits(value: u32, bits: usize) -> u32 {
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
#[derive(Debug)]
pub struct Block {
    pub last: bool,
    pub r#type: BlockType,
    pub litlen_tree: LitLenTree,
    pub distance_tree: DistanceTree,
    pub codlen_tree: CodlenTree,
}
impl Default for Block {
    fn default() -> Self {
        Self { last: Default::default(), r#type: Default::default(), litlen_tree: HuffmanTree::default(), distance_tree: HuffmanTree::default(), codlen_tree: HuffmanTree::default() }
    }
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
            let symbol = self.codlen_tree.decode_symbol(reader)? as u8;

            match symbol {
                0..=15 => all_codelengths.push(symbol),
                16 => {
                    let repeat = reader.read_bits(2)? as u8 + 3;
                    let prev = *all_codelengths.last().unwrap_or(&0);
                    for _ in 0..repeat { all_codelengths.push(prev); }
                }
                17 => {
                    let repeat = reader.read_bits(3)? as u8 + 3;
                    all_codelengths.resize(all_codelengths.len() + repeat as usize, 0);
                }
                18 => {
                    let repeat = reader.read_bits(7)? as u8 + 11;
                    all_codelengths.resize(all_codelengths.len() + repeat as usize, 0);
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
