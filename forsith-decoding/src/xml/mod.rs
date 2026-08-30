use core::str::FromStr;
use std::borrow::Cow;
use anyhow::{Context, Result, bail, ensure};
use derive_more::IsVariant;
use forsith_shared::interner::StringInterner;

mod parser;
use parser::XmlParser;

mod tree;
use tree::XmlTree;

#[cfg(test)]
mod tests;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Prolog {
    version: XmlVersion,
    encoding: Encoding,
    standalone: bool,
}

#[derive(Debug)]
pub struct XmlDocument<'a> {
    prolog: Prolog,
    pub interner: StringInterner<'a>,
    pub tree: XmlTree
}

#[derive(Debug, Default, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct XmlVersion(usize);
impl FromStr for XmlVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        ensure!(s.starts_with("1."), "Invalid XML version: {s}");

        let version_num = s[2..].parse::<usize>()
            .map_err(|_| anyhow::anyhow!("Too large minor XML version: {s}"))?;

        Ok(Self(version_num))
    }
}

impl XmlVersion {
    #[allow(dead_code)]
    #[must_use]
    pub const fn minor(&self) -> usize {self.0}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, IsVariant)]
pub enum Encoding {
    #[default]
    Utf8,
    Utf16LE,
    Utf16BE,
}

impl Encoding {
    fn decode(self, data: Cow<'_, [u8]>) -> Result<Cow<'_, str>> {
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

    fn from_str(s: &str) -> Result<Self> {
        if s.eq_ignore_ascii_case("utf-8") {return Ok(Self::Utf8);}
        else if s.eq_ignore_ascii_case("utf-16") | s.eq_ignore_ascii_case("utf-16le") {return Ok(Self::Utf16LE);}
        else if s.eq_ignore_ascii_case("utf-16be") {return Ok(Self::Utf16BE);}

        bail!("Unknown or unsupported encoding: {s}");
    }
}

impl XmlDocument<'_> {
    #[must_use]
    pub const fn prolog(&self) -> &Prolog {&self.prolog}

    pub fn parse(data: Cow<'_, [u8]>) -> Result<Self> {Self::parse_with_interner(data, StringInterner::default())}

    pub fn parse_with_interner<'a>(data: Cow<'_, [u8]>, mut interner: StringInterner<'a>) -> Result<XmlDocument<'a>> {
        let encoding = Encoding::identify_in_xml(&data);
        let data = encoding.decode(data)?;
        let mut data = XmlParser::from(&*data);

        let prolog = data.prolog(&mut interner).with_context(|| format!("Failed to parse prolog, error was likely located around {}", data.current_source_position()))?;
        ensure!(prolog.encoding == encoding, "Encoding mismatch: prolog specifies {:?}, but detected {encoding:?}", prolog.encoding);

        let tree = XmlTree::parse(&mut data, &mut interner).with_context(|| format!("Failed to parse elements, error was likely located around {}", data.current_source_position()))?;

        Ok(XmlDocument {
            prolog,
            interner,
            tree,
        })
    }

}
