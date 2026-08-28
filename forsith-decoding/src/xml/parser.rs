use std::{io::BufRead};
use anyhow::{bail, Result};
use derive_more::{Deref, DerefMut, IsVariant};
use forsith_shared::{buffers::CursorString, interner::{InternedString, StringInterner}};

use super::{Encoding, Prolog, XmlVersion};

#[derive(Debug, Deref, DerefMut)]
pub struct XmlParser<'input>(CursorString<'input>);

impl<'input> From<&'input str> for XmlParser<'input> {
    fn from(s: &'input str) -> Self {
        Self(CursorString::from(s))
    }
}

pub enum ParsedContentItem {
    Tag(ParsedTag),
    Misc,
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Deref)]
pub struct ParsedTag {
    pub name: InternedString,
    pub attributes: Vec<(InternedString, InternedString)>,
    #[deref]
    pub kind: TagKind,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, IsVariant)]
pub enum TagKind {
    Opening,
    Closing,
    Empty,
}

impl XmlParser<'_> {
    pub fn current_source_position(&self) -> String {
        let cursor = self.cursor();

        let (line, col) = self.line_col(cursor);

        format!("{line}:{col}")
    }

    pub fn expect(&mut self, expected: &str) -> Result<()> {
        let actual = self.peek(expected.len());
        if actual != expected {bail!("Expected '{expected}', found '{actual}'")}
        self.consume(expected.len());
        Ok(())
    }

    pub fn whitespaces(&mut self) {
        let whitespaces = self.remaining_str()
            .find(|c: char| !c.is_whitespace())
            .unwrap_or_else(|| self.remaining_str().len());
        self.consume(whitespaces);
    }

    pub fn misc(&mut self) -> Result<()> {
        self.whitespaces();

        while self.comment()?.is_some() || self.processing_instruction()?.is_some() {
            self.whitespaces();
        }

        Ok(())
    }

    pub fn comment(&mut self) -> Result<Option<()>> {
        if self.expect("<!--").is_err() {return Ok(None)}

        let comment_end = self.remaining_str().find("-->")
            .ok_or_else(|| anyhow::anyhow!("Unterminated comment"))?;

        self.consume(comment_end + 3);

        Ok(Some(()))
    }

    #[allow(unreachable_code)]
    pub fn processing_instruction(&mut self) -> Result<Option<()>> {
        if self.expect("<?").is_err() {return Ok(None)}

        let pi_end = self.remaining_str().find("?>")
            .ok_or_else(|| anyhow::anyhow!("Unterminated processing instruction"))?;

        let instruction = self.take(pi_end);
        todo!("Handle processing instruction: {instruction:?}");

        self.consume(2);
        Ok(Some(()))
    }

    pub fn xml_decl(&mut self, prolog: &mut Prolog) -> Result<Option<()>> {
        if self.expect("<?xml").is_err() {return Ok(None);}

        prolog.version = self.version_info()?;
        self.encoding_decl()?.map(|x| prolog.encoding = x);
        self.standalone_decl()?.map(|x| prolog.standalone = x);

        self.whitespaces();

        self.expect("?>")?;

        Ok(Some(()))
    }

    pub fn name(&mut self, interner: &mut StringInterner) -> Result<InternedString> {
        let name_end = self.remaining_str().find(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .unwrap_or_else(|| self.remaining_str().len());

        Ok(interner.interned(self.take(name_end)))
    }

    pub fn content_item(&mut self, interner: &mut StringInterner) -> Result<ParsedContentItem> {
        if self.comment()?.is_some() || self.processing_instruction()?.is_some() {
            return Ok(ParsedContentItem::Misc)
        }

        if self.expect("<").is_err() {return Ok(ParsedContentItem::None)}
        let mut kind = if self.expect("/").is_ok() {TagKind::Closing} else {TagKind::Opening};

        let name = self.name(interner)?;

        self.whitespaces();

        let mut attributes = Vec::new();
        while !matches!(self.peek(1), "/" | ">") {
            let attr_name = self.name(interner)?;
            self.eq()?;
            let attr_value = self.qouted_string()?;
            attributes.push((attr_name, interner.interned(attr_value)));

            self.whitespaces();
        }

        if kind.is_closing() && !attributes.is_empty() {bail!("Closing tag cannot have attributes")}

        if kind.is_opening() && self.expect("/>").is_ok() {kind = TagKind::Empty;}
        else {self.expect(">")?;}

        Ok(ParsedContentItem::Tag(ParsedTag { name, attributes, kind }))
    }

    pub fn prolog(&mut self, _interner: &mut StringInterner) -> Result<Prolog> {
        let mut prolog = Prolog::default();

        self.xml_decl(&mut prolog)?;

        self.misc()?;

        if self.doctype_decl()?.is_some() {
            self.misc()?;
        }

        Ok(prolog)
    }

    pub fn string_until_tag(&mut self) -> Option<&str> {
        let tag_start = self.remaining_str().find('<')
            .unwrap_or_else(|| self.remaining_str().len());

        if tag_start == 0 {return None}

        let str = self.take(tag_start).trim();

        match str {"" => None, str => Some(str)}
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn doctype_decl(&mut self) -> Result<Option<()>> {
        if self.expect("<!DOCTYPE").is_ok() {
            todo!("Handle doctype declaration");
        }

        Ok(None)
    }

    pub fn eq(&mut self) -> Result<()> {
        self.whitespaces();

        self.expect("=")?;

        self.whitespaces();

        Ok(())
    }

    pub fn qouted_string(&mut self) -> Result<&str> {
        match self.peek(1) {
            "'" | "\"" => (),
            c => bail!("Expected quote, found '{c:?}'"),
        }

        let qoute = self.take(1).chars().next().unwrap();

        let terminating_qoute = self.remaining_str().find(qoute)
            .ok_or_else(|| anyhow::anyhow!("Unterminated string"))?;

        Ok(&self.take(terminating_qoute + 1)[..terminating_qoute])
    }

    pub fn declaration(&mut self, key: &'static str) -> Result<Option<&str>> {
        self.whitespaces();

        if self.expect(key).is_err() {return Ok(None)}

        self.eq()?;

        self.qouted_string().map(Some)
    }

    pub fn version_info(&mut self) -> Result<XmlVersion> {
        let s = self.declaration("version")?
            .ok_or_else(|| anyhow::anyhow!("Missing version declaration"))?;

        XmlVersion::from_str(s)
    }

    pub fn encoding_decl(&mut self) -> Result<Option<Encoding>> {
        self.declaration("encoding")?
            .map(Encoding::from_str)
            .transpose()
    }

    pub fn standalone_decl(&mut self) -> Result<Option<bool>> {
        self.declaration("standalone")?
            .map(|s| match s {
                "yes" => Ok(true),
                "no" => Ok(false),
                _ => bail!("Invalid standalone declaration: {s:?}"),
            })
            .transpose()
    }
}
