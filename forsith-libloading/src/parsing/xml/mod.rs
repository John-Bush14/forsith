use std::{borrow::Cow, io::Read};
use anyhow::{Result, bail};

pub struct XmlDocument {
    encoding: Encoding,
}

enum Encoding {
    Utf8,
    Utf16LE,
    Utf16BE,
}

impl Encoding {
    fn decode<'a>(&self, data: Cow<'a, [u8]>) -> Result<Cow<'a, str>> {
        match self {
            Self::Utf8 => {
                match data {
                    Cow::Borrowed(slice) => Ok(str::from_utf8(slice)?.into()),
                    Cow::Owned(vec) => Ok(String::from_utf8(vec)?.into()),
                }
            }
            Self::Utf16LE => Ok(String::from_utf16le(&data)?.into()),
            Self::Utf16BE => Ok(String::from_utf16be(&data)?.into()),
        }
    }
}

impl XmlDocument {
    pub fn parse(data: Cow<'_, [u8]>) -> Result<XmlDocument> {
        let start = &data[..4.min(data.len())];

        let encoding = match start {
            [0xEF, 0xBB, 0xBF, ..] | [0x3C, 0x3F, 0x78, 0x6D] => Encoding::Utf8,
            [0xFF, 0xFE, ..] | [0x3C, 0x00, 0x3F, 0x00] => Encoding::Utf16LE,
            [0xFE, 0xFF, ..] | [0x00, 0x3C, 0x00, 0x3F] => Encoding::Utf16BE,
            _ => {
                println!("Assuming UTF-8, might be wrong: {start:?}");
                Encoding::Utf8
            }
        };

        let data = encoding.decode(data)?;

        Ok(XmlDocument {
            encoding
        })
    }
}
