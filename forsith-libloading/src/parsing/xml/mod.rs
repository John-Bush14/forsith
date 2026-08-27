use std::{borrow::Cow, num::NonZero};
use anyhow::{Result, bail, ensure};
use derive_more::IsVariant;
use forsith_shared::interner::{InternedString, StringInterner};

mod parser;
use parser::XmlParser;

use crate::parsing::xml::parser::{ParsedTag, TagKind};

#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
pub struct Prolog {
    version: XmlVersion,
    encoding: Encoding,
    standalone: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum XmlNode {
    Tag(XmlTagNode),
    Attribute(InternedString, InternedString),
    Text(InternedString),
}

#[derive(Debug, PartialEq, Eq)]
pub struct XmlTagNode {
    name: InternedString,
    attributes: usize,
    next_sibling: Option<NonZero<usize>>
}

#[derive(Debug)]
pub struct XmlDocument<'a> {
    prolog: Prolog,
    interner: StringInterner<'a>,
    content: Vec<XmlNode>
}

#[derive(Debug, Default)]
pub struct XmlVersion(usize);
impl XmlVersion {
    pub fn from_str(s: &str) -> Result<Self> {
        ensure!(s.starts_with("1."), "Invalid XML version: {s}");

        let version_num = s[2..].parse::<usize>()
            .map_err(|_| anyhow::anyhow!("Too large minor XML version: {s}"))?;

        Ok(Self(version_num))
    }

    #[allow(dead_code)]
    pub const fn minor(&self) -> usize {self.0}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, IsVariant)]
enum Encoding {
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
    pub fn parse(data: Cow<'_, [u8]>) -> Result<Self> {Self::parse_with_interner(data, StringInterner::default())}

    pub fn parse_with_interner<'a>(data: Cow<'_, [u8]>, mut interner: StringInterner<'a>) -> Result<XmlDocument<'a>> {
        let encoding = Encoding::identify_in_xml(&data);
        let data = encoding.decode(data)?;
        let mut data = XmlParser::from(&*data);

        let prolog = data.prolog(&mut interner)?;
        ensure!(prolog.encoding == encoding, "Encoding mismatch: prolog specifies {:?}, but detected {encoding:?}", prolog.encoding);

        let mut doc = XmlDocument {
            prolog,
            interner,
            content: Vec::new(),
        };

        doc.parse_elements(&mut data)?;

        Ok(doc)
    }

    fn push_tag(&mut self, element: ParsedTag) {
        self.content.push(XmlNode::Tag(XmlTagNode {
            name: element.name,
            attributes: element.attributes.len(),
            next_sibling: None,
        }));
        self.content.extend(element.attributes.into_iter().map(|(name, value)| XmlNode::Attribute(name, value)));
    }

    fn parse_elements(&mut self, parser: &mut XmlParser) -> Result<()> {
        let root = parser.tag(&mut self.interner)?.ok_or_else(|| anyhow::anyhow!("No root element found"))?;
        let root_name = root.name;
        ensure!(!root.kind.is_closing(), "Root element cannot be a closing tag");
        self.push_tag(root);

        let closer = self.parse_element_content(parser)?;
        ensure!(closer == root_name, "Root element not closed properly: expected </{}>, found </{}>", self.interner.resolve(root_name), self.interner.resolve(closer));

        parser.misc()?;

        ensure!(parser.remaining_str().is_empty(), "Unexpected content after root element");

        Ok(())
    }

    fn parse_element_content(&mut self, parser: &mut XmlParser) -> Result<InternedString> {
        let mut prev_tag: Option<usize> = None;

        loop {
            parser.string_until_tag()?.map(|text| {
                let text_interned = self.interner.interned(text);
                self.content.push(XmlNode::Text(text_interned));
            });

            let tag = parser.tag(&mut self.interner)?.ok_or_else(|| anyhow::anyhow!("Unterminated tag"))?;

            if tag.kind.is_closing() {return Ok(tag.name);}

            if let Some(prev_element) = prev_tag {
                let cur = NonZero::new(self.content.len());

                match self.content[prev_element] {
                    XmlNode::Tag(ref mut prev) => {
                        prev.next_sibling = cur;
                    }
                    _ => bail!("prev_element in XmlDocument content is not an Element node"),
                }
            }

            prev_tag = Some(self.content.len());
            let (kind, name) = (tag.kind, tag.name);
            self.push_tag(tag);

            if kind.is_opening() {
                let closer = self.parse_element_content(parser)?;
                ensure!(closer == name, "Element not closed properly: expected </{}>, found </{}>", self.interner.resolve(name), self.interner.resolve(closer));
            }
        }
    }
}
