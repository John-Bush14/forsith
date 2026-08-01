use crate::{DecodingError, decompression::BitReader, };

#[derive(Debug)]
#[repr(C)]
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

    #[inline(always)]
    const fn symbol(&self) -> u16 {
        (self.0 & u16::MAX as u32) as u16
    }
    #[inline(always)]
    const fn subtable_index(&self) -> usize {self.symbol() as usize}

    #[inline(always)]
    const fn colen(&self) -> u8 {
        (self.0 >> 24) as u8
    }
    #[inline(always)]
    const fn subtable_bits(&self) -> u8 {self.colen()}

    const fn code(&self) -> u16 {
        (self.0 >> 16) as u16
    }

    const fn _is_symbol(&self) -> bool {self.0 >> 16 & 0b11 == 1}
    const fn is_subtable(&self) -> bool {self.0 >> 16 & 0b11 == 2}

    #[allow(unused)]
    const fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl<const MAX_COLEN: u8, const MAX_ROOT_COLEN: u8, const MAX_SUBTABLE_ENTRIES: usize> HuffmanTree<MAX_COLEN, MAX_ROOT_COLEN, MAX_SUBTABLE_ENTRIES>
where
    [(); (1 << MAX_ROOT_COLEN as usize) + MAX_SUBTABLE_ENTRIES]:,
    [(); MAX_COLEN as usize + 1]:
{
    pub const fn default() -> Self {
        Self {
            table: [Entry::empty(); (1 << MAX_ROOT_COLEN as usize) + MAX_SUBTABLE_ENTRIES],
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

            if self.table[root].subtable_index() != self.generation || !self.table[root].is_subtable() {
                self.table[root] = Entry::new_subtable(self.generation, subcolen);
            } else {
                self.table[root] = Entry::new_subtable(self.generation, self.table[root].subtable_bits().max(subcolen));
            }

            longcodes[self.next_subtable] = Entry::new_longcode(symbol, code as u16);
            self.next_subtable += 1;
        }

        if MAX_SUBTABLE_ENTRIES == 0 {return Ok(());}

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

    #[inline(always)]
    pub fn decode_symbol<R: BitReader>(&self, reader: &mut R) -> u16 {
        let code = reader.peek_bits(self.root_bits);

        let mut entry = self.table[code as usize];

        if MAX_SUBTABLE_ENTRIES != 0 && entry.is_subtable() {
            let subtable_bits = reader.peek_bits_nobranch(entry.subtable_bits() + MAX_ROOT_COLEN) >> MAX_ROOT_COLEN;
            entry = self.table[entry.subtable_index() + subtable_bits as usize];
        }

        reader.consume_bits(entry.colen());
        entry.symbol()
    }
}

#[inline(always)]
const fn reverse_bits(value: u32, bits: usize) -> u32 {
    value.reverse_bits() >> (32 - bits)
}
