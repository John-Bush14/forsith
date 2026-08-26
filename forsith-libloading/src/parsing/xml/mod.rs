use std::io::BufRead;

pub struct XmlDocument;

impl XmlDocument {
    pub fn parse(xml: impl BufRead) -> Result<XmlDocument, String> {
        todo!()
    }
}
