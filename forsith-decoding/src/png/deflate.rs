use const_for::const_for;
use derive_more::IsVariant;

use crate::{DecodingError, parsing::BitReader, decompression::{HuffmanTree}};

const CODE_LENGTH_ORDER: [u8; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

type LitLenTree = HuffmanTree<15, 9, MAX_LITLEN_SUBTABLE_ENTIES>;
type DistanceTree = HuffmanTree<15, 9, MAX_DISTANCE_SUBTABLE_ENTRIES>;
type CodlenTree = HuffmanTree<7, 7, 0>;

const MAX_LITLEN_SUBTABLE_ENTIES: usize = 340;
const MAX_DISTANCE_SUBTABLE_ENTRIES: usize = 80;

pub const MAX_BACKREF_LEN: usize = 258;

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

#[inline(always)]
pub fn decode_length<R: BitReader>(symbol: u16, reader: &mut R) -> u16 {
    let (base, extra) = LENGTH_TABLE[(symbol - 257) as usize];
    base + reader.read_bits_nobranch(extra) as u16
}

#[inline(always)]
pub fn decode_distance<R: BitReader>(code: u16, reader: &mut R) -> u16 {
    let (base, extra) = DISTANCE_TABLE[code as usize];
    base + reader.read_bits_nobranch(extra) as u16
}

#[derive(Debug, Default, PartialEq, Eq, IsVariant)]
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
        self.last = reader.read_bits(1) == 1;
        self.load_compression_type(reader)?;

        if self.r#type == BlockType::CompressedDynamic {
            self.load_trees(reader)?;
        }

        Ok(())
    }

    fn load_trees<R: BitReader>(&mut self, reader: &mut R) -> Result<(), DecodingError> {
        let hlit: u16 = reader.read_bits(5) as u16 + 257;
        let hdist: u16 = reader.read_bits(5) as u16 + 1;
        let hclen: u16 = reader.read_bits(4) as u16 + 4;

        let mut codlen_codelengths = vec![0u8; 19];
        for (i, colen) in reader.iterate_bits::<3>().take(hclen as usize).enumerate() {
            let index = CODE_LENGTH_ORDER[i] as usize;
            codlen_codelengths[index] = colen as u8;
        }
        self.codlen_tree.load(&codlen_codelengths)?;

        // Decode hlit + hdist code lengths, expanding RLE symbols 16/17/18
        let total = hlit as usize + hdist as usize;
        let mut all_codelengths = Vec::with_capacity(total);

        while all_codelengths.len() < total {
            let symbol = self.codlen_tree.decode_symbol(reader) as u8;

            match symbol {
                0..=15 => {all_codelengths.push(symbol);},
                _ => {
                    let (extra_bits, base, use_prev) = match symbol {
                        16 => (2, 3, true),
                        17 => (3, 3, false),
                        18 => (7, 11, false),
                        _ => unreachable!(),
                    };

                    let prev = if !use_prev {0} else {*all_codelengths.last().unwrap()};

                    let repeat = base + reader.read_bits(extra_bits) as usize;
                    all_codelengths.resize(all_codelengths.len() + repeat, prev);
                }
            }
        }

        self.litlen_tree.load(&all_codelengths[..hlit as usize])?;
        self.distance_tree.load(&all_codelengths[hlit as usize..])?;

        Ok(())
    }

    fn load_compression_type<R: BitReader>(&mut self, reader: &mut R) -> Result<(), DecodingError> {
        match reader.read_bits(2) {
            0 => {
                let alignment_bits = reader.remaining_bits() % 8;
                reader.consume_bits(alignment_bits);

                let len = reader.read_bits(16) as u16;
                let nlen = reader.read_bits(16) as u16;

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
