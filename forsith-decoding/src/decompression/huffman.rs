use derive_more::IsVariant;
use num_enum::IntoPrimitive;
use crate::{DecodingError, parsing::BitRead};

#[derive(Debug, Clone)]
pub struct HuffmanTree<const MAX_COLEN: u8, const MAX_ROOT_COLEN: u8, const MAX_SUBTABLE_ENTRIES: usize>
where
    [(); (1 << MAX_ROOT_COLEN as usize) + MAX_SUBTABLE_ENTRIES]:,
    [(); MAX_COLEN as usize + 1]:
{
    table: [Entry; (1 << MAX_ROOT_COLEN as usize) + MAX_SUBTABLE_ENTRIES],
    root_bits: u8,
    sub_bits: u8,
    next_subtable: usize,
    generation: usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
struct Entry(u32);

#[derive(IntoPrimitive, Debug, Clone, Copy, PartialEq, Eq, IsVariant)]
#[repr(u32)]
enum EntryTag {
    Empty = 0,
    Symbol = 1,
    Subtable = 2,
}

impl Entry {
    const VALUE_MASK: u32 = 0xFFFF;
    const TAG_SHIFT: u32 = 16;
    const TAG_MASK: u32 = 0b11;
    const META_SHIFT: u32 = 24;

    pub const EMPTY: Self = Self(0);

    #[inline(always)]
    const fn new_symbol(symbol: u16, colen: u8) -> Self {
        Self(
            symbol as u32
                | ((EntryTag::Symbol as u32) << Self::TAG_SHIFT)
                | ((colen as u32) << Self::META_SHIFT),
        )
    }

    /// `index` is the offset from the end of the root table.
    #[inline(always)]
    #[allow(clippy::cast_possible_truncation)]
    const fn new_subtable(index: usize, bits: u8) -> Self {
        Self(
            index as u32
                | ((EntryTag::Subtable as u32) << Self::TAG_SHIFT)
                | ((bits as u32) << Self::META_SHIFT),
        )
    }

    /// Not used during decoding.
    #[inline(always)]
    const fn new_longcode(symbol: u16, code: u16) -> Self {
        Self(symbol as u32 | ((code as u32) << 16))
    }

    #[inline(always)]
    const fn value(self) -> u16 {
        (self.0 & Self::VALUE_MASK) as u16
    }
    const fn symbol(self) -> u16 {self.value()}
    const fn subtable_index(self) -> usize {self.value() as usize}

    #[inline(always)]
    const fn meta(self) -> u8 {
        (self.0 >> Self::META_SHIFT) as u8
    }
    const fn colen(self) -> u8 {self.meta()}
    const fn subtable_bits(self) -> u8 {self.meta()}

    #[inline(always)]
    const fn code(self) -> u16 {
        (self.0 >> Self::TAG_SHIFT) as u16
    }

    #[inline(always)]
    const fn tag(self) -> EntryTag {
        match (self.0 >> Self::TAG_SHIFT) & Self::TAG_MASK {
            0 => EntryTag::Empty,
            1 => EntryTag::Symbol,
            2 => EntryTag::Subtable,
            _ => unreachable!(),
        }
    }
}

impl<const MAX_COLEN: u8, const MAX_ROOT_COLEN: u8, const MAX_SUBTABLE_ENTRIES: usize> HuffmanTree<MAX_COLEN, MAX_ROOT_COLEN, MAX_SUBTABLE_ENTRIES>
where
    [(); (1 << MAX_ROOT_COLEN as usize) + MAX_SUBTABLE_ENTRIES]:,
    [(); MAX_COLEN as usize + 1]:
{
    pub const fn default() -> Self {
        Self {
            table: [Entry::EMPTY; (1 << MAX_ROOT_COLEN as usize) + MAX_SUBTABLE_ENTRIES],
            root_bits: 0,
            sub_bits: 0,
            next_subtable: 0,
            generation: 0,
        }
    }

    pub const fn load(&mut self, code_lengths: &[u8]) -> Result<(), DecodingError> {
        let (mut colen_counts, max_colen) = Self::get_colen_counts(code_lengths);

        if max_colen > MAX_COLEN {return Err(DecodingError::InvalidCodeLength(max_colen))}

        self.root_bits = max_colen.min(MAX_ROOT_COLEN);

        if MAX_SUBTABLE_ENTRIES > 0 {
            self.sub_bits = max_colen.saturating_sub(MAX_ROOT_COLEN);
            self.generation = (self.generation + 1) & ((1 << MAX_ROOT_COLEN) - 1);
        }

        if colen_counts[2] == 0 && colen_counts[1] == 1 {
            colen_counts[0] = 1; // make single symbol have code 1
        }

        let first_codes = self.generate_first_codes(&colen_counts);

        self.generate_table(code_lengths, first_codes);

        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    const fn generate_table(&mut self, code_lengths: &[u8], mut next_code: [u16; MAX_COLEN as usize + 1]) {
        let mut longcodes = [Entry::EMPTY; MAX_SUBTABLE_ENTRIES];
        self.next_subtable = 0;

        assert!(code_lengths.len() <= u16::MAX as usize);

        let mut i = 0usize;
        while i < code_lengths.len() {
            let colen = code_lengths[i];
            let symbol = i as u16; i += 1;

            if colen == 0 {continue;}

            let code = next_code[colen as usize];
            next_code[colen as usize] += 1;

            let code = reverse_bits(code, colen as _);

            if MAX_SUBTABLE_ENTRIES == 0 || colen <= MAX_ROOT_COLEN {
                let filler = 1 << (self.root_bits - colen);

                let mut i = 0;
                while i < filler {
                    self.table[(code as usize) | (i << colen)]  = Entry::new_symbol(symbol, colen);

                    i += 1;
                }

                continue;
            }

            let subcolen = colen - MAX_ROOT_COLEN;
            let root = (code & ((1 << MAX_ROOT_COLEN) - 1)) as usize;

            if self.table[root].subtable_index() != self.generation || !self.table[root].tag().is_subtable() {
                self.table[root] = Entry::new_subtable(self.generation, subcolen);
            } else {
                self.table[root] = Entry::new_subtable(self.generation, self.table[root].subtable_bits().max(subcolen));
            }

            longcodes[self.next_subtable] = Entry::new_longcode(symbol, code);
            self.next_subtable += 1;
        }

        if MAX_SUBTABLE_ENTRIES == 0 {return}

        let longcodes_len = self.next_subtable;
        self.next_subtable = 1 << MAX_ROOT_COLEN;

        let mut i = 0;
        while i < longcodes_len {
            let entry = longcodes[i]; i += 1;

            let (symbol, code) = (entry.symbol(), entry.code());
            let colen = code_lengths[symbol as usize];

            let root = code & ((1 << MAX_ROOT_COLEN) - 1);
            let subcode = code >> MAX_ROOT_COLEN;

            let root_entry = &mut self.table[root as usize];

            if root_entry.subtable_index() == self.generation {
                *root_entry = Entry::new_subtable(self.next_subtable, root_entry.subtable_bits());
                self.next_subtable += 1 << root_entry.subtable_bits();
            }

            let subcolen = colen - MAX_ROOT_COLEN;

            let subtable_start = root_entry.subtable_index();
            let subtable_bits = root_entry.subtable_bits();

            let filler = 1 << (subtable_bits - subcolen);

            let mut i = 0;
            while i < filler {
                self.table[subtable_start + ((subcode as usize) | (i << subcolen))] = Entry::new_symbol(symbol, colen);

                i += 1;
            }
        }
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

    const fn generate_first_codes(&self, colen_counts: &[u16; MAX_COLEN as usize + 1]) -> [u16; MAX_COLEN as usize + 1] {
        let mut first_codes = [0u16; MAX_COLEN as usize + 1];

        let mut i = 0;
        while i < (self.root_bits + self.sub_bits) as usize {
            i += 1;

            first_codes[i] = (first_codes[i - 1] + colen_counts[i - 1]) << 1;
        }

        first_codes
    }

    #[inline(always)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn decode_symbol<R: BitRead>(&self, reader: &mut R) -> u16 {
        let code = reader.peek_bits(self.root_bits);

        let mut entry = self.table[code as usize];

        if MAX_SUBTABLE_ENTRIES != 0 && entry.tag().is_subtable() {
            let subtable_bits = reader.peek_bits_nobranch(entry.subtable_bits() + MAX_ROOT_COLEN) >> MAX_ROOT_COLEN;
            entry = self.table[entry.subtable_index() + subtable_bits as usize];
        }

        reader.consume_bits(entry.colen());
        entry.symbol()
    }
}

#[inline(always)]
const fn reverse_bits(value: u16, bits: usize) -> u16 {
    value.reverse_bits() >> (16 - bits)
}
