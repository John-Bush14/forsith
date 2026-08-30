use std::num::NonZero;
use forsith_shared::interner::{InternedString, StringInterner};
use anyhow::{Result, bail, ensure, Context};
use crate::xml::{parser::{ParsedContentItem, ParsedTag, XmlParser}, tree::{XmlRootNode, XmlTagNode, XmlTree, XmlTreeNode}};

#[derive(Debug, PartialEq, Eq)]
pub struct XmlTreeBuilder {
    root: XmlRootNode,
    subtree: Vec<XmlTreeNode>
}

impl From<XmlTreeBuilder> for XmlTree {
    fn from(builder: XmlTreeBuilder) -> Self {
        Self {
            root: builder.root,
            root_subtree: builder.subtree.into_boxed_slice(),
        }
    }
}

impl XmlTreeBuilder {
    fn push_tag(&mut self, element: ParsedTag) {
        self.subtree.push(XmlTreeNode::Tag(XmlTagNode {
            name: element.name,
            attributes: element.attributes.len(),
            next_sibling: None,
        }));
        self.subtree.extend(element.attributes.into_iter().map(XmlTreeNode::Attribute));
    }

    pub fn parse(parser: &mut XmlParser, interner: &mut StringInterner) -> Result<Self> {
        let ParsedContentItem::Tag(root) = parser.content_item(interner)? else {bail!("No root tag found")};
        ensure!(!root.kind.is_closing(), "Root element cannot be a closing tag");

        let mut builder = Self {
            root: XmlRootNode {
                name: root.name,
                attributes: root.attributes.into_iter().map(XmlTreeNode::Attribute).collect(),
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
