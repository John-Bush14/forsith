use super::ImageParser;
use std::path::Path;


pub struct JpgParser {}


impl ImageParser for JpgParser {
    fn parse(_path: &Path) -> Result<((u32, u32), u64, Vec<u8>), Box<dyn std::error::Error>> {todo!();}
}
