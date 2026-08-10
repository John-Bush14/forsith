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
    fn iterate_bits(&mut self, n: u8) -> BitIterator<'_, Self> where Self: Sized {
        BitIterator {
            reader: self,
            bits: n
        }
    }
}

pub struct BitIterator<'a, R: BitReader> {
    reader: &'a mut R,
    bits: u8
}
impl<R: BitReader> Iterator for BitIterator<'_, R> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.reader.read_bits(self.bits))
    }
}
