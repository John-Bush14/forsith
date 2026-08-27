use std::{borrow::Cow, io::Read};
use anyhow::{Result, bail};
use forsith_shared::interner::StringInterner;

pub struct XmlDocument<'a> {
    encoding: Encoding,
    interner: StringInterner<'a>
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

    fn identify_in_xml(data: &[u8]) -> Self {
        match &data[..4.min(data.len())] {
            [0xEF, 0xBB, 0xBF, ..] | [0x3C, 0x3F, 0x78, 0x6D] => Self::Utf8,
            [0xFF, 0xFE, ..] | [0x3C, 0x00, 0x3F, 0x00] => Self::Utf16LE,
            [0xFE, 0xFF, ..] | [0x00, 0x3C, 0x00, 0x3F] => Self::Utf16BE,
            start => {
                println!("Assuming UTF-8, might be wrong: {start:?}");
                Self::Utf8
            }
        }
    }
}

impl XmlDocument<'_> {
    pub fn parse(data: Cow<'_, [u8]>) -> Result<Self> {
        let encoding = Encoding::identify_in_xml(&data);
        let data = encoding.decode(data)?;
        let mut interner = StringInterner::default();



        Ok(XmlDocument {
            encoding,
            interner,
        })
    }
}
