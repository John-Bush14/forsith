use std::path::Path;
use std::ffi::OsStr;


mod jpg;


pub trait ImageParser {
    fn parse(file: &Path) -> Result<((u32, u32), u64, Vec<u8>), Box<dyn std::error::Error>>;
}


#[allow(unreachable_code, dead_code, non_snake_case)]
pub fn parse_image(file: &Path) -> Result<((u32, u32), u64, Vec<u8>), Box<dyn std::error::Error>> {
    let file_extension: Option<&OsStr> = file.extension();

    return match file_extension {
        None => panic!("parse_image: path {:?} doesn't have file extension?", file),
        Some(extension) => match extension.to_str().unwrap() {
            "jpg" | "jpeg" => return jpg::JpgParser::parse(file),
            _ => panic!("unsupported format (parse_image): {:?}", file)
        }
    }
}
