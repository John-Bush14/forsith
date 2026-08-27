use crate::parsing::XmlDocument;

fn assert_parsed(xml: &str, expected: ()) {
    let mut document = XmlDocument::parse(xml.as_bytes().into()).expect("Failed to parse XML");

    panic!("Parsed document: {:?}", document.content);

    // TODO: Implement a proper comparison between the parsed document and the expected structure.
}

#[test]
fn only_root() {
    assert_parsed("<root></root>", ());
}

#[test]
fn prolog_root() {
    assert_parsed("<?xml version=\"1.0\" encoding=\"UTF-8\" ?><root></root>", ());
}
