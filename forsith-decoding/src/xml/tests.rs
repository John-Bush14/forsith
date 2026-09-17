use forsith_shared::interner::StringInterner;

use crate::xml::XmlNode;

use super::XmlDocument;

fn parse(xml: &str) -> (XmlDocument, StringInterner<'_>) {
    let mut interner = StringInterner::default();
    let document = XmlDocument::parse_with_interner(xml.as_bytes().into(), &mut interner)
        .expect("XML should parse successfully");
    (document, interner)
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
fn parses_simple_nested_elements() {
    let (document, interner) = parse("<root><child><leaf /></child></root>");
    let root = document.tree.root();

    assert_eq!(interner.resolve(root.name()), "root");
    let mut children = root.children();

    match children.next() {
        Some(XmlNode::Tag(tag)) => {
            assert_eq!(interner.resolve(tag.name()), "child");
            let mut descendants = tag.children();
            match descendants.next() {
                Some(XmlNode::Tag(leaf)) => assert_eq!(interner.resolve(leaf.name()), "leaf"),
                other => panic!("Expected a leaf tag, got {other:?}"),
            }
            assert!(descendants.next().is_none());
        }
        other => panic!("Expected a child tag, got {other:?}"),
    }

    assert!(children.next().is_none());
}

#[test]
fn parses_attributes_and_text_nodes() {
    let (document, mut interner) =
        parse(r#"<root id="root-id"><child kind="leaf">value</child></root>"#);

    let root = document.tree.root();

    assert_eq!(
        root.attribute(interner.interned("id")),
        Some(interner.interned("root-id"))
    );

    let children: Vec<_> = root.children().collect();
    assert_eq!(children.len(), 1);

    match &children[0] {
        XmlNode::Tag(tag) => {
            assert_eq!(interner.resolve(tag.name()), "child");
            assert_eq!(
                tag.attribute(interner.interned("kind")),
                Some(interner.interned("leaf"))
            );

            let descendants: Vec<_> = tag.children().collect();
            assert_eq!(descendants.len(), 1);

            match &descendants[0] {
                XmlNode::Text(text) => assert_eq!(interner.resolve(*text), "value"),
                other @ XmlNode::Tag(_) => panic!("Expected text node, got {other:?}"),
            }
        }
        other @ XmlNode::Text(_) => panic!("Expected child tag, got {other:?}"),
    }
}

#[test]
fn parses_prolog_and_utf8_document() {
    let (document, mut interner) =
        parse(r#"<?xml version="1.0" encoding="UTF-8"?><root attr="value"></root>"#);

    let root = document.tree.root();

    assert_eq!(interner.resolve(root.name()), "root");
    assert_eq!(
        root.attribute(interner.interned("attr")),
        Some(interner.interned("value"))
    );
}

#[test]
fn parses_utf16_documents() {
    let xml = "<?xml version=\"1.0\" encoding=\"UTF-16\"?><root attr=\"value\"/>";
    let bytes: Vec<u8> = xml.encode_utf16().flat_map(u16::to_le_bytes).collect();

    let (document, mut interner) =
        XmlDocument::parse(bytes.into()).expect("UTF-16 XML should parse");

    let root = document.tree.root();

    assert_eq!(interner.resolve(root.name()), "root");
    assert_eq!(
        root.attribute(interner.interned("attr")),
        Some(interner.interned("value"))
    );
    assert!(root.children().next().is_none());
}
