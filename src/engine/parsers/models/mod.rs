use crate::vulkan::vertex::Vertex;
use std::ffi::OsStr;
use std::path::Path;


mod obj;


pub trait ModelParser {
    fn parse(file: &Path) -> Result<Vec<Vec<Vertex>>, Box<dyn std::error::Error>>;
}



#[allow(unreachable_code, dead_code, non_snake_case)]
pub fn parse_model(file: &Path) -> Result<Vec<Vec<Vertex>>, Box<dyn std::error::Error>> {
    let file_extension: Option<&OsStr> = file.extension();

    return match file_extension {
        None => panic!("parse_model: path {:?} doesn't have file extension?", file),
        Some(extension) => match extension.to_str().unwrap() {
            "obj" => return obj::ObjParser::parse(file),
            _ => panic!("unsupported format (parse_image): {:?}", file)
        }
    }
}
