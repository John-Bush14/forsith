use crate::{DecodingError, bit::BitRead, decompression::HuffmanTree};
use forsith_proc::{Deref, IsVariant};
use forsith_shared::{bit::BitReadExt, buffers::CursorVec, const_for};
use std::io::Read;

const CODE_LENGTH_ORDER: [u8; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

type LitLenTree = HuffmanTree<15, 9, MAX_LITLEN_SUBTABLE_ENTIES>;
type DistanceTree = HuffmanTree<15, 9, MAX_DISTANCE_SUBTABLE_ENTRIES>;
type CodlenTree = HuffmanTree<7, 7, 0>;

const MAX_LITLEN_SUBTABLE_ENTIES: usize = 340;
const MAX_DISTANCE_SUBTABLE_ENTRIES: usize = 80;

pub const MAX_BACKREF_LEN: usize = 258;

// (base, extra_bits) for length symbols 257..=285
const LENGTH_TABLE: [(u16, u8); 29] = {
    let mut table = [(0u16, 0u8); 29];

    let mut base = 3;
    const_for!(i in 0..28u8 => {
        let extra_bits = (i.saturating_sub(4)) / 4;

        table[i as usize] = (base, extra_bits);
        base += 1 << extra_bits;
    });

    table[28] = (258, 0);

    table
};

// (base, extra_bits) for distance codes 0..=29
const DISTANCE_TABLE: [(u16, u8); 30] = {
    let mut table = [(0u16, 0u8); 30];

    let mut base = 1;
    const_for!(i in 0..30u8 => {
        let extra_bits = (i.saturating_sub(2)) / 2;

        table[i as usize] = (base, extra_bits);
        base += 1 << extra_bits;
    });

    table
};

pub const STATIC_LITLEN_TREE: LitLenTree = {
    let mut tree = HuffmanTree::default();

    let mut lengths = [0u8; 288];

    const_for!(i in 0..144 => {lengths[i] = 8;});
    const_for!(i in 144..256 => {lengths[i] = 9;});
    const_for!(i in 256..280 => {lengths[i] = 7;});
    const_for!(i in 280..288 => {lengths[i] = 8;});

    if let Err(_e) = tree.load(&lengths) {
        panic!("loading static litlen tree failed")
    }

    tree
};

pub const STATIC_DISTANCE_TREE: DistanceTree = {
    let mut tree = HuffmanTree::default();

    let lengths = [5u8; 30];

    if let Err(_e) = tree.load(&lengths) {
        panic!("loading static distance tree failed")
    }

    tree
};

#[derive(Debug)]
pub struct DeflateDecoder {
    pub litlen_tree: LitLenTree,
    pub distance_tree: DistanceTree,
    pub codlen_tree: CodlenTree,
    pub block: Block,
}

impl Default for DeflateDecoder {
    fn default() -> Self {
        Self {
            litlen_tree: LitLenTree::default(),
            distance_tree: DistanceTree::default(),
            codlen_tree: CodlenTree::default(),
            block: Block::default(),
        }
    }
}

impl DeflateDecoder {
    pub const fn cur_block(&self) -> &Block {
        &self.block
    }

    pub fn next_block(&mut self, reader: &mut impl BitRead) -> Result<(), DecodingError> {
        if self.block.last {
            self.block.ty = BlockType::Finished;
            return Ok(());
        }

        self.block.load_block(reader)?;

        if self.block.is_compressed_dynamic() {
            self.load_trees(reader)?;
        }

        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    fn load_trees<R: BitRead>(&mut self, reader: &mut R) -> Result<(), DecodingError> {
        let hlit: usize = reader.read_bits(5) as usize + 257;
        let hdist: u16 = reader.read_bits(5) as u16 + 1;
        let hclen: u16 = reader.read_bits(4) as u16 + 4;

        let mut codlen_codelengths = vec![0u8; 19];
        for (i, colen) in reader.iterate_bits::<3>().take(hclen as usize).enumerate() {
            let index = CODE_LENGTH_ORDER[i] as usize;
            codlen_codelengths[index] = colen as u8;
        }
        self.codlen_tree.load(&codlen_codelengths)?;

        // Decode hlit + hdist code lengths, expanding RLE symbols 16/17/18
        let total = hlit + hdist as usize;
        let mut all_codelengths = Vec::with_capacity(total);

        while all_codelengths.len() < total {
            let symbol = u8::try_from(self.codlen_tree.decode_symbol(reader)).unwrap();

            if let 0..=15 = symbol {
                all_codelengths.push(symbol);
            } else {
                let (extra_bits, base, use_prev) = match symbol {
                    16 => (2, 3, true),
                    17 => (3, 3, false),
                    18 => (7, 11, false),
                    _ => unreachable!(),
                };

                let prev = if use_prev {
                    *all_codelengths.last().unwrap()
                } else {
                    0
                };

                let repeat = base + reader.read_bits(extra_bits) as usize;
                all_codelengths.resize(all_codelengths.len() + repeat, prev);
            }
        }

        self.litlen_tree.load(&all_codelengths[..hlit])?;
        self.distance_tree.load(&all_codelengths[hlit..])?;

        Ok(())
    }

    pub(crate) fn decode_untill_full(
        &mut self,
        src: &mut impl BitReadExt,
        buf: &mut CursorVec<u8>,
        mut drain: impl FnMut(&mut CursorVec<u8>) -> Result<bool, DecodingError>,
    ) -> Result<(), DecodingError> {
        while !self.decode_chunk(src, buf, &mut drain)? {}

        Ok(())
    }

    fn decode_chunk(
        &mut self,
        src: &mut impl BitReadExt,
        buf: &mut CursorVec<u8>,
        drain: &mut impl FnMut(&mut CursorVec<u8>) -> Result<bool, DecodingError>,
    ) -> Result<bool, DecodingError> {
        match self.cur_block().ty {
            BlockType::Uncompressed(len) => self.read_uncompressed_chunk(len, src, buf, drain),
            BlockType::CompressedFixed => self.read_compressed_chunk::<true>(src, buf, drain),
            BlockType::CompressedDynamic => self.read_compressed_chunk::<false>(src, buf, drain),
            BlockType::Finished => Ok(true),
        }
    }

    #[cold]
    fn read_uncompressed_chunk(
        &mut self,
        len: u16,
        src: &mut impl BitReadExt,
        buf: &mut CursorVec<u8>,
        drain: &mut impl FnMut(&mut CursorVec<u8>) -> Result<bool, DecodingError>,
    ) -> Result<bool, DecodingError> {
        src.unconsume_bitbuf();

        let mut decoded_bytes = 0;

        loop {
            let chunk_len = (len - decoded_bytes).min(buf.remaining().saturating_truncate());

            buf.read_from(src, chunk_len as usize)?;

            decoded_bytes += chunk_len;

            if decoded_bytes >= len || !drain(buf)? {
                break;
            }
        }

        if len == decoded_bytes {
            self.next_block(src)?;
            Ok(false)
        } else {
            self.set_uncompressed_len(len - decoded_bytes);
            Ok(true)
        }
    }

    #[cold]
    fn read_compressed_chunk<const STATIC: bool>(
        &mut self,
        src: &mut impl BitReadExt,
        buf: &mut CursorVec<u8>,
        drain: &mut impl FnMut(&mut CursorVec<u8>) -> Result<bool, DecodingError>,
    ) -> Result<bool, DecodingError> {
        loop {
            let buffer_full = buf.cursor() + MAX_BACKREF_LEN >= buf.capacity();

            if core::hint::unlikely(buffer_full) && !drain(buf)? {
                return Ok(true);
            }

            if self.decode_compressed::<STATIC>(src, buf) {
                self.next_block(src)?;
                return Ok(false);
            }
        }
    }

    #[inline(always)]
    pub fn decode_compressed<const STATIC: bool>(
        &self,
        reader: &mut impl BitRead,
        buf: &mut CursorVec<u8>,
    ) -> bool {
        let (litlen_tree, distance_tree) = if STATIC {
            (&STATIC_LITLEN_TREE, &STATIC_DISTANCE_TREE)
        } else {
            (&self.litlen_tree, &self.distance_tree)
        };

        let symbol = litlen_tree.decode_symbol(reader);

        if symbol < 256 {
            buf.write_fast_single(u8::try_from(symbol).unwrap());
        } else if std::hint::unlikely(symbol == 256) {
            return true;
        } else {
            let length = decode_length(symbol, reader);
            let dist_code = distance_tree.decode_symbol(reader);
            let distance = decode_distance(dist_code, reader);

            if distance as usize >= length as usize {
                copy_backreference(buf, length as usize, distance as usize);
            } else if distance == 1 {
                let fill = buf.get_ref()[buf.cursor() - 1];
                buf.take_mut_slice(length as usize).fill(fill);
            } else {
                for _ in 0..length - distance {
                    let byte = buf.get_ref()[buf.cursor() - distance as usize];
                    buf.write_fast_single(byte);
                }

                copy_backreference(buf, distance as usize, distance as usize);
            }
        }

        false
    }

    pub fn set_uncompressed_len(&mut self, len: u16) {
        #[cfg(debug_assertions)]
        assert!(matches!(self.block.ty, BlockType::Uncompressed(_)));

        self.block.ty = BlockType::Uncompressed(len);
    }

    #[allow(clippy::needless_pass_by_ref_mut, clippy::unused_self)]
    pub fn open_stream(&mut self, reader: &mut impl Read) -> Result<usize, DecodingError> {
        let cmf = reader.read_le::<u8>()?;
        let flg = reader.read_le::<u8>()?;
        // reader.align()?;

        let compression_method = cmf & 0b0000_1111;
        let compression_info = (cmf & 0b1111_0000) >> 4;
        let dict = flg & 0b0010_0000 == 0b0010_0000;

        if compression_method != 8
            || compression_info > 7
            || dict
            || !(u16::from(cmf) * 256 + u16::from(flg)).is_multiple_of(31)
        {
            return Err(DecodingError::InvalidZlibHeader(cmf, flg));
        }

        Ok(1 << (compression_info + 8))
    }

    pub const fn is_finished(&self) -> bool {
        matches!(self.block.ty, BlockType::Finished)
    }
}

#[inline]
fn copy_backreference(buf: &mut CursorVec<u8>, length: usize, distance: usize) {
    let i = buf.cursor();

    buf.get_mut()
        .copy_within(i - distance..i - distance + length, i);

    buf.consume(length);
}

#[inline(always)]
#[allow(clippy::cast_possible_truncation)]
pub fn decode_length<R: BitRead>(symbol: u16, reader: &mut R) -> u16 {
    let (base, extra) = LENGTH_TABLE[(symbol - 257) as usize];
    base + reader.read_bits_nobranch(extra) as u16
}

#[inline(always)]
#[allow(clippy::cast_possible_truncation)]
pub fn decode_distance<R: BitRead>(code: u16, reader: &mut R) -> u16 {
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
#[derive(Debug, Deref, Default)]
pub struct Block {
    pub last: bool,
    #[deref]
    pub ty: BlockType,
}

impl Block {
    pub fn load_block<R: BitRead>(&mut self, reader: &mut R) -> Result<(), DecodingError> {
        self.last = reader.read_bits(1) == 1;
        self.load_compression_type(reader)?;

        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    fn load_compression_type<R: BitRead>(&mut self, reader: &mut R) -> Result<(), DecodingError> {
        match reader.read_bits(2) {
            0 => {
                let alignment_bits = reader.remaining_bits() % 8;
                reader.consume_bits(alignment_bits);

                let len = reader.read_bits(16) as u16;
                let nlen = reader.read_bits(16) as u16;

                if len != !nlen {
                    return Err(DecodingError::BlockLengthMismatch(len, nlen));
                }

                self.ty = BlockType::Uncompressed(len);
            }
            1 => self.ty = BlockType::CompressedFixed,
            2 => self.ty = BlockType::CompressedDynamic,
            _ => return Err(DecodingError::ReservedCompressionMethod),
        }

        Ok(())
    }
}
