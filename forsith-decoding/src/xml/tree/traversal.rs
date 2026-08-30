use derive_more::Deref;
use forsith_shared::interner::InternedString;

use crate::xml::tree::{AttributeNode, XmlTree, XmlTreeNode};


#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct XmlSubTree<'a>(&'a [XmlTreeNode]);
impl<'a> From<&'a [XmlTreeNode]> for XmlSubTree<'a> {
    fn from(slice: &'a [XmlTreeNode]) -> Self {Self(slice)}
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

impl XmlTree {
    fn root_subtree(&self) -> XmlSubTree<'_> {XmlSubTree::from(&*self.root_subtree)}

    pub(crate) fn root_tag(&self) -> XmlTag<'_> {
        XmlTag { name: self.root.name, attributes: &self.root.attributes, subtree: self.root_subtree()}
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

                let tree_start = self.current + 1 + tag.attributes;
                let subtree = self.tree.subtree(tree_start, tag.next_sibling.map_or(tree_start, std::convert::Into::into));

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

