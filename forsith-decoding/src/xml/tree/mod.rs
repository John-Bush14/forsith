use forsith_shared::{error::Result, interner::{InternedString, StringInterner}};
use crate::xml::parser::XmlParser;

mod creation;
use creation::XmlTreeBuilder;

mod traversal;
use traversal::XmlTag;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct XmlRootNode {
    pub(crate) name: InternedString,
    pub(crate) attributes: Box<[XmlTreeNode]>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum XmlTreeNode {
    Tag(XmlTagNode),
    Attribute(AttributeNode),
    Text(InternedString),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AttributeNode {
    key: InternedString,
    val: InternedString
}
impl AttributeNode {
    pub const fn key(&self) -> InternedString {self.key}
    pub const fn val(&self) -> InternedString {self.val}
    pub(crate) const fn new(key: InternedString, val: InternedString) -> Self {Self {key, val}}
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct XmlTagNode {
    pub(crate) name: InternedString,
    pub(crate) attributes: usize,
    /// excluding attributes and the index of the next non-child node
    pub(crate) len: usize
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct XmlTree {
    pub(crate) root: XmlRootNode,
    pub(crate) root_subtree: Box<[XmlTreeNode]>
}

impl XmlTree {
    pub(crate) fn parse(parser: &mut XmlParser, interner: &mut StringInterner) -> Result<Self> {
        XmlTreeBuilder::parse(parser, interner).map(std::convert::Into::into)
    }

    pub fn root(&self) -> XmlTag<'_> {self.root_tag()}
}
