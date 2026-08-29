use core::num::NonZero;
use std::ops::{Deref};
use derive_more::Deref;
use forsith_shared::interner::{InternedString, StringInterner};
use anyhow::{Result, Context, bail, ensure};
use crate::xml::parser::{ParsedContentItem, ParsedTag, XmlParser};

#[derive(Debug, PartialEq, Eq)]
pub enum XmlTreeNode {
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
    pub(crate) attributes: Box<[XmlTreeNode]>,
}

#[derive(Deref)]
#[derive(Debug, PartialEq, Eq)]
pub struct XmlTag<'a> {
    name: InternedString,
    attributes: &'a [XmlTreeNode],
    #[deref]
    subtree: XmlSubTree<'a>,
}
impl XmlTag<'_> {
    pub const fn name(&self) -> InternedString {self.name}
    pub fn attributes(&self) -> impl Iterator<Item = &AttributeNode> {
        self.attributes.iter().map(|a| match a {XmlTreeNode::Attribute(a) => a, _ => panic!("XmlTag's attributes contained non-attribute")})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum XmlNode<'a> {
    Tag(XmlTag<'a>),
    Text(InternedString),
}

pub struct XmlDescendants<'a> {
    tree: XmlSubTree<'a>,
    current: usize
}

impl<'a> Iterator for XmlDescendants<'a> {
    type Item = XmlNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.tree.0.len() {return None}

        let node = match &self.tree.0[self.current] {
            XmlTreeNode::Tag(tag) => {
                let attributes = self.tree.slice(self.current + 1, tag.attributes);
                assert!(attributes.iter().all(|n| matches!(n, XmlTreeNode::Attribute(_))), "Non attributes found in nodes attributes slice");

                let subtree = self.tree.subtree(self.current+1+tag.attributes, tag.next_sibling.map_or(self.current+1+tag.attributes, std::convert::Into::into));

                XmlNode::Tag(XmlTag {
                    name: tag.name,
                    attributes,
                    subtree
                })
            }
            XmlTreeNode::Text(text) => XmlNode::Text(*text),
            XmlTreeNode::Attribute(_) => panic!("Unexpected attribute node in XmlDescendants iterator"),
        };

        self.current += 1;

        Some(node)
    }
}

impl<'a> XmlSubTree<'a> {
    pub const fn descendants(self) -> XmlDescendants<'a> {
        XmlDescendants {
            tree: self,
            current: 0
        }
    }

    fn slice(self, start: usize, length: usize) -> &'a [XmlTreeNode] {
        &self.0[start..start + length]
    }
    fn subtree(self, start: usize, end: usize) -> Self {
        XmlSubTree(&self.0[start..end])
    }
}

#[derive(Debug, PartialEq, Eq)]
struct XmlTreeBuilder {
    root: XmlRootNode,
    subtree: Vec<XmlTreeNode>
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
    pub(crate) subtree: Box<[XmlTreeNode]>
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct XmlSubTree<'a>(&'a [XmlTreeNode]);

impl XmlTree {
    pub fn root_subtree(&self) -> XmlSubTree<'_> {XmlSubTree(&self.subtree)}

    pub fn root(&self) -> XmlTag<'_> {
        XmlTag { name: self.root.name, attributes: &self.root.attributes, subtree: self.root_subtree()}
    }

    pub(crate) fn parse(parser: &mut XmlParser, interner: &mut StringInterner) -> Result<Self> {
        XmlTreeBuilder::parse(parser, interner).map(std::convert::Into::into)
    }
}

impl XmlTreeBuilder {
    fn push_tag(&mut self, element: ParsedTag) {
        self.subtree.push(XmlTreeNode::Tag(XmlTagNode {
            name: element.name,
            attributes: element.attributes.len(),
            next_sibling: None,
        }));
        self.subtree.extend(element.attributes.into_iter().map(|(name, value)| XmlTreeNode::Attribute((name, value))));
    }

    fn parse(parser: &mut XmlParser, interner: &mut StringInterner) -> Result<Self> {
        let ParsedContentItem::Tag(root) = parser.content_item(interner)? else {bail!("No root tag found")};
        ensure!(!root.kind.is_closing(), "Root element cannot be a closing tag");

        let mut builder = Self {
            root: XmlRootNode {
                name: root.name,
                attributes: root.attributes.iter().map(|&a| XmlTreeNode::Attribute(a)).collect(),
            },
            subtree: Vec::new(),
        };

        if root.kind.is_opening() {
            builder.parse_element_content(parser, interner, root.name)?;
        }

        parser.misc()?;

        ensure!(parser.remaining_str().is_empty(), "Unexpected content after root element");

        Ok(builder)
    }

    fn handle_chardata(&mut self, parser: &mut XmlParser, interner: &mut StringInterner) {
        parser.string_until_tag().map(|text| {
            let text_interned = interner.interned(text);
            self.subtree.push(XmlTreeNode::Text(text_interned));
        });
    }

    fn update_prev_sibling(&mut self, prev_sibling: usize) {
        let cur = NonZero::new(self.subtree.len());

        match self.subtree[prev_sibling] {
            XmlTreeNode::Tag(ref mut prev) => {
                prev.next_sibling = cur;
            }
            _ => panic!("prev_sibling in XmlDocument content is not an Element node"),
        }
    }

    fn parse_element_content(&mut self, parser: &mut XmlParser, interner: &mut StringInterner, parent: InternedString) -> Result<()> {
        let mut prev_tag: Option<usize> = None;

        (|| {loop {
            self.handle_chardata(parser, interner);

            let tag = match parser.content_item(interner)? {
                ParsedContentItem::Misc => continue,
                ParsedContentItem::Tag(tag) => {tag}
                ParsedContentItem::None => bail!("Unterminated tag"),
            };

            if tag.kind.is_closing() {
                ensure!(tag.name == parent, "Element not closed properly: expected </{}>, found </{}>", interner.resolve(parent), interner.resolve(tag.name));

                return Ok(());
            }

            if let Some(prev_sibling) = prev_tag {self.update_prev_sibling(prev_sibling)}
            prev_tag = Some(self.subtree.len());

            let (kind, name) = (tag.kind, tag.name);
            self.push_tag(tag);

            if kind.is_opening() {
                self.parse_element_content(parser, interner, name)?;
            }
        }})().with_context(|| format!("Failed to parse content of <{}>", interner.resolve(parent)))
    }
}
