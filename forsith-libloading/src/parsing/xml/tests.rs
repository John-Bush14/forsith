use anyhow::ensure;
use forsith_shared::interner::StringInterner;

use crate::parsing::{XmlDocument, xml::XmlNode};

fn assert_parsed(xml: &str, expected_nodes: (Vec<XmlNode>, StringInterner)) {
    let (expected, interner) = expected_nodes;

    let document = XmlDocument::parse_with_interner(xml.as_bytes().into(), interner).expect("Failed to parse XML");

    assert_eq!(document.content, expected, "Parsed document does not match expected structure: {:?} != {:?}", document.content, expected);
}

macro_rules! expected_nodes {
    ($(
        $(($name:literal, $att:literal, $sib:expr))?
        $({$key:literal = $val:literal})?
        $($str:literal)?
    ),*) => {{
        let mut interner = forsith_shared::interner::StringInterner::default();

        (vec![
            $(
                $(
                    XmlNode::Tag(crate::parsing::xml::XmlTagNode {
                        name: interner.interned($name),
                        attributes: $att,
                        next_sibling: $sib,
                    }),
                )?
                $(
                    XmlNode::Attribute(interner.interned($key), interner.interned($val)),
                )?
                $(
                    XmlNode::Text(InternedString::from($str)),
                )?
            )*
        ], interner)
    }};
}

#[test]
fn only_root() {
    assert_parsed("<root></root>", expected_nodes!(
        ("root", 0, None)
    ));
}

#[test]
fn only_root_attribute() {
    assert_parsed("<root attribute=\"test\"></root>", expected_nodes!(
        ("root", 1, None),
        {"attribute" = "test"}
    ));
}

#[test]
fn prolog_root() {
    assert_parsed("<?xml version=\"1.0\" encoding=\"UTF-8\" ?><root></root>", expected_nodes!(
        ("root", 0, None)
    ));
}
