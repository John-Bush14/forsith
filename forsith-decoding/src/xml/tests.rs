use forsith_shared::interner::{InternedString, StringInterner};

use crate::xml::tree::{AttributeNode, XmlRootNode, XmlTree};

use super::{XmlDocument, tree::{XmlTreeNode, XmlTagNode}};

type ExpectedNodes<'a> = (InternedString, Vec<AttributeNode>, Box<[XmlTreeNode]>, StringInterner<'a>);
fn assert_parsed(xml: &str, expected_nodes: ExpectedNodes) {
    let (root_name, root_attributes, expected, mut interner) = expected_nodes;

    let expected = XmlTree {
        root: XmlRootNode {
            name: root_name,
            attributes: root_attributes.into_iter().map(XmlTreeNode::Attribute).collect(),
        },
        root_subtree: expected,
    };

    let document = XmlDocument::parse_with_interner(xml.as_bytes().into(), &mut interner).expect("Failed to parse XML");

    assert_eq!(document.tree, expected, "Parsed document does not match expected structure: {:?} != {:?}", document.tree, expected);
}

macro_rules! expected_nodes {
    (
        $root:literal: [$( $rootkey:literal = $rootval:literal ),*]:
        $(
            $(($name:literal, $att:literal, $sib:expr))?
            $({$key:literal = $val:literal})?
            $($str:literal)?
        ,)*
    ) => {{
        #[allow(unused_mut)]
        let mut interner = forsith_shared::interner::StringInterner::default();

        (interner.interned($root), vec![$(AttributeNode::new(interner.interned($rootkey), interner.interned($rootval))),*], Box::new([
            $(
                $(
                    XmlTreeNode::Tag(XmlTagNode {
                        name: interner.interned($name),
                        attributes: $att,
                        len: $sib,
                    }),
                )?
                $(
                    XmlNode::Attribute((interner.interned($key), interner.interned($val))),
                )?
                $(
                    XmlNode::Text(InternedString::from($str)),
                )?
            )*
        ]), interner)
    }};
}

#[should_panic(expected = "No root tag found")]
#[test]
fn no_root() {
    XmlDocument::parse(b"".into()).unwrap();
}

#[should_panic(expected = "No root tag found")]
#[test]
fn prolog_no_root() {
    XmlDocument::parse(b"<?xml version=\"1.0\"?>".into()).unwrap();
}

#[test]
fn simple_nest() {
    assert_parsed("<root><nested></nested></root>", expected_nodes!(
        "root": []:
        ("nested", 0, 0),
    ));
}

#[test]
fn simple_siblings() {
    assert_parsed("<root><sibling></sibling><sibling></sibling><sibling></sibling></root>", expected_nodes!(
        "root": []:
        ("sibling", 0, 0),
        ("sibling", 0, 0),
        ("sibling", 0, 0),
    ));
}

#[test]
fn only_root() {
    assert_parsed("<root></root>", expected_nodes!(
        "root": []:
    ));
}

#[test]
fn only_root_attribute() {
    assert_parsed(r#"<root attribute="test"></root>"#, expected_nodes!(
        "root": ["attribute" = "test"]:
    ));
}

#[test]
fn prolog_only_root() {
    assert_parsed(r#"<?xml version="1.0" encoding="UTF-8" ?><root></root>"#, expected_nodes!(
        "root": []:
    ));
}
