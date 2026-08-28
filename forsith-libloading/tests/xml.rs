use forsith_libloading::parsing::XmlDocument;

#[test]
fn main() {
    XmlDocument::parse(include_bytes!("assets/test.xml").into()).unwrap();
}
