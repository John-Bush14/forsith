use std::io::Read;

use const_for::const_for;

use crate::{DecodingError, read_exact_array};

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

#[derive(Debug)]
pub struct CRCReader<R: Read> {
    pub reader: R,
    pub crc: u32
}

impl<R: Read> CRCReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            crc: INIT
        }
    }

    pub fn normal_reader(&mut self) -> &mut R {&mut self.reader}

    pub fn validate_crc(&mut self) -> Result<(), DecodingError> {
        let stored_crc = u32::from_be_bytes(read_exact_array::<4,_>(self.normal_reader())?);

        self.crc = !self.crc;

        if self.crc != stored_crc {
            return Err(DecodingError::CRCMismatch(self.crc, stored_crc));
        }

        self.crc = INIT;

        Ok(())
    }

    pub fn update_crc(&mut self, buf: &[u8]) {
        for b in buf {
            self.crc = CRC_TABLE[((self.crc ^ *b as u32) & 0xff) as usize] ^ (self.crc >> 8);
        }
    }
}

impl<R: Read> Read for CRCReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.reader.read(buf)?;
        self.update_crc(&buf[..read]);
        Ok(read)
    }
}
