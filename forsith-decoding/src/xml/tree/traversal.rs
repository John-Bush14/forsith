use derive_more::Deref;
use forsith_shared::interner::InternedString;

use crate::xml::tree::{AttributeNode, XmlTree, XmlTreeNode};


#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct XmlSubTree<'a>(&'a [XmlTreeNode]);
impl<'a> From<&'a [XmlTreeNode]> for XmlSubTree<'a> {
    fn from(slice: &'a [XmlTreeNode]) -> Self {Self(slice)}
}

#[derive(Deref)]
#[derive(Debug, PartialEq, Eq, Clone)]
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

    pub fn attribute(&self, key: InternedString) -> Option<InternedString> {
        self.attributes().find(|a| a.key() == key).map(AttributeNode::val)
    }
}

impl XmlTree {
    fn root_subtree(&self) -> XmlSubTree<'_> {XmlSubTree::from(&*self.root_subtree)}

    pub(crate) fn root_tag(&self) -> XmlTag<'_> {
        XmlTag { name: self.root.name, attributes: &self.root.attributes, subtree: self.root_subtree()}
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum XmlNode<'a> {
    Tag(XmlTag<'a>),
    Text(InternedString),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct XmlChildren<'a> {
    tree: XmlSubTree<'a>,
    current: usize
}

impl<'a> Iterator for XmlChildren<'a> {
    type Item = XmlNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.tree.0.len() {return None}

        let node = match &self.tree.0[self.current] {
            XmlTreeNode::Text(text) => {
                self.current += 1;
                XmlNode::Text(*text)
            },
            XmlTreeNode::Tag(treetag) => {
                let tag = self.tree.tag(self.current);

                self.current += 1 + treetag.attributes + treetag.len;

                XmlNode::Tag(tag)
            },
            XmlTreeNode::Attribute(_) => panic!("Unexpected attribute node in XmlChildren iterator"),
        };

        Some(node)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct XmlDescendants<'a> {
    tree: XmlSubTree<'a>,
    current: usize
}

impl<'a> Iterator for XmlDescendants<'a> {
    type Item = XmlNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.tree.0.len() {return None}

        let node = match &self.tree.0[self.current] {
            XmlTreeNode::Tag(_) => {
                let tag = self.tree.tag(self.current);

                self.current += 1 + tag.attributes.len();

                XmlNode::Tag(tag)
            }
            XmlTreeNode::Text(text) => {
                self.current += 1;
                XmlNode::Text(*text)
            },
            XmlTreeNode::Attribute(_) => panic!("Unexpected attribute node in XmlDescendants iterator"),
        };

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

    pub const fn children(self) -> XmlChildren<'a> {
        XmlChildren {
            tree: self,
            current: 0
        }
    }

    fn slice(self, start: usize, length: usize) -> &'a [XmlTreeNode] {
        &self.0[start..start + length]
    }
    fn subtree(self, start: usize, length: usize) -> Self {
        XmlSubTree(self.slice(start, length))
    }

    fn tag(self, index: usize) -> XmlTag<'a> {
        let XmlTreeNode::Tag(tag) = &self.0[index] else {panic!("tree.tag(index) called for index containing non-tag node")};

        let attributes = self.slice(index + 1, tag.attributes);

        let start = index + 1 + tag.attributes;
        let subtree = self.subtree(start, tag.len);

        XmlTag {
            name: tag.name,
            attributes,
            subtree
        }
    }
}

