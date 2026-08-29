use core::num::NonZero;
use std::ops::{Deref};
use forsith_shared::interner::{InternedString, StringInterner};
use anyhow::{Result, Context, bail, ensure};
use crate::xml::parser::{ParsedContentItem, ParsedTag, XmlParser};

#[derive(Debug, PartialEq, Eq)]
pub enum XmlNode {
    Tag(XmlTagNode),
    Attribute(AttributeNode),
    Text(InternedString),
}

type AttributeNode = (InternedString, InternedString);

#[derive(Debug, PartialEq, Eq)]
pub struct XmlTagNode {
    pub(crate) name: InternedString,
    pub(crate) attributes: usize,
    pub(crate) next_sibling: Option<NonZero<usize>>
}
#[derive(Debug, PartialEq, Eq)]
pub struct XmlRootNode {
    pub(crate) name: InternedString,
    pub(crate) attributes: Box<[AttributeNode]>,
}

#[derive(Debug, PartialEq, Eq)]
struct XmlTreeBuilder {
    root: XmlRootNode,
    subtree: Vec<XmlNode>
}
impl From<XmlTreeBuilder> for XmlTree {
    fn from(builder: XmlTreeBuilder) -> Self {
        Self {
            root: builder.root,
            subtree: builder.subtree.into_boxed_slice(),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct XmlTree {
    pub(crate) root: XmlRootNode,
    pub(crate) subtree: Box<[XmlNode]>
}

impl Deref for XmlTree {
    type Target = [XmlNode];

    fn deref(&self) -> &Self::Target {&self.subtree}
}

#[derive(Debug, PartialEq, Eq)]
pub struct XmlSubTree<'a>(&'a [XmlNode]);

impl XmlTree {
    pub fn as_subtree(&self) -> XmlSubTree<'_> {XmlSubTree(&self.subtree)}

    pub fn parse(parser: &mut XmlParser, interner: &mut StringInterner) -> Result<Self> {
        XmlTreeBuilder::parse(parser, interner).map(std::convert::Into::into)
    }
}

impl XmlTreeBuilder {
    fn push_tag(&mut self, element: ParsedTag) {
        self.subtree.push(XmlNode::Tag(XmlTagNode {
            name: element.name,
            attributes: element.attributes.len(),
            next_sibling: None,
        }));
        self.subtree.extend(element.attributes.into_iter().map(|(name, value)| XmlNode::Attribute((name, value))));
    }

    fn parse(parser: &mut XmlParser, interner: &mut StringInterner) -> Result<Self> {
        let ParsedContentItem::Tag(root) = parser.content_item(interner)? else {bail!("No root tag found")};
        ensure!(!root.kind.is_closing(), "Root element cannot be a closing tag");

        let mut builder = Self {
            root: XmlRootNode {
                name: root.name,
                attributes: root.attributes.into_boxed_slice(),
            },
            subtree: Vec::new(),
        };

        if root.kind.is_opening() {
            let closer = builder.parse_element_content(parser, interner).with_context(|| format!("Failed to parse content of root <{}>", interner.resolve(root.name)))?;

            ensure!(closer == root.name, "Root element not closed properly: expected </{}>, found </{}>", interner.resolve(root.name), interner.resolve(closer));
        }

        parser.misc()?;

        ensure!(parser.remaining_str().is_empty(), "Unexpected content after root element");

        Ok(builder)
    }

    fn parse_element_content(&mut self, parser: &mut XmlParser, interner: &mut StringInterner) -> Result<InternedString> {
        let mut prev_tag: Option<usize> = None;

        loop {
            parser.string_until_tag().map(|text| {
                let text_interned = interner.interned(text);
                self.subtree.push(XmlNode::Text(text_interned));
            });

            let tag = match parser.content_item(interner)? {
                ParsedContentItem::Misc => continue,
                ParsedContentItem::Tag(tag) => {tag}
                ParsedContentItem::None => bail!("Unterminated tag"),
            };

            if tag.kind.is_closing() {return Ok(tag.name);}

            if let Some(prev_element) = prev_tag {
                let cur = NonZero::new(self.subtree.len());

                match self.subtree[prev_element] {
                    XmlNode::Tag(ref mut prev) => {
                        prev.next_sibling = cur;
                    }
                    _ => bail!("prev_element in XmlDocument content is not an Element node"),
                }
            }

            prev_tag = Some(self.subtree.len());
            let (kind, name) = (tag.kind, tag.name);
            self.push_tag(tag);

            if kind.is_opening() {
                let closer = self.parse_element_content(parser, interner).with_context(|| format!("Failed to parse content of <{}>", interner.resolve(name)))?;
                ensure!(closer == name, "Element not closed properly: expected </{}>, found </{}>", interner.resolve(name), interner.resolve(closer));
            }
        }
    }
}

impl XmlSubTree<'_> {
}
