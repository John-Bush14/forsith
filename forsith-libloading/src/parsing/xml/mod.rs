use std::io::Read;
use anyhow::{Result, bail};

pub struct XmlDocument;

enum Encoding {
    Utf8,
    Utf16LE,
    Utf16BE,
}

impl XmlDocument {
    pub fn parse(xml: Vec<u8>) -> Result<XmlDocument> {
        let start = &xml[..4.min(xml.len())];

        let encoding = match start {
            [0xEF, 0xBB, 0xBF, _] | [0x3C, 0x3F, 0x78, 0x6D] => Encoding::Utf8,
            [0xFF, 0xFE, _, _] | [0x3C, 0x00, 0x3F, 0x00] => Encoding::Utf16LE,
            [0xFE, 0xFF, _, _] | [0x00, 0x3C, 0x00, 0x3F] => Encoding::Utf16BE,
            _ => {
                println!("Assuming UTF-8, might be wrong: {start:?}");
                Encoding::Utf8
            }
        };

        let xml = match encoding {
            Encoding::Utf8 => String::from_utf8(xml)?,
            Encoding::Utf16LE => String::from_utf16le(&xml)?,
            Encoding::Utf16BE => String::from_utf16be(&xml)?,
        };

        todo!()
    }
}
