use std::io::Read;

use const_for::const_for;
use crate::{DecodingError, png::chunkreader::ChunkReader, read_exact_array};


pub const POLY: u32 = 0xedb88320;
const CRC_TABLE: [u32; 256] = const {
    let mut table = [0u32; 256];

    const_for!(n in 0..255+1 => {
        let mut c = n as u32;
        const_for!(_ in 0..8 => {
            if c & 1 == 1 {
                c = POLY ^ (c >> 1);
                continue;
            }
            c >>= 1;
        });
        table[n as usize] = c
    });

    table
};
const INIT: u32 = 0xFFFF_FFFF;


impl<R: Read> ChunkReader<R> {
    pub fn validate_crc(&mut self) -> Result<(), DecodingError> {
        let stored_crc = u32::from_be_bytes(read_exact_array::<4,_>(self.normal_reader())?);

        self.crc = !self.crc;

        if self.crc != stored_crc {
            return Err(DecodingError::CRCMismatch(self.crc, stored_crc));
        }

        Ok(())
    }

    pub fn update_crc(&mut self, buf: &[u8]) {
        for b in buf {
            self.crc = CRC_TABLE[((self.crc ^ *b as u32) & 0xff) as usize] ^ (self.crc >> 8);
        }
    }

    pub fn init_crc(&mut self) {
        self.crc = INIT;
    }
}
