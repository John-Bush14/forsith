use crate::vulkan::vertex::Vertex;
use std::ffi::OsStr;
use std::path::Path;


mod obj;


pub trait ModelParser {
    fn parse(file: &Path) -> Result<(Vec<Mesh>, Vec<[f32;3]>, Vec<[f32;2]>), Box<dyn std::error::Error>>;
}


#[derive(Default, Clone)]
pub struct Mesh {
    pub name: String,
    pub vertex_indices: Vec<u32>,
    pub texcoord_indices: Vec<u32>
}


impl Mesh {pub(crate) fn new(name: String) -> Mesh {
    return Mesh {vertex_indices: vec!(), texcoord_indices: vec!(), name}
}}


#[allow(unreachable_code, dead_code, non_snake_case)]
pub fn parse_model(file: &Path) -> Result<(Vec<Mesh>, Vec<[f32;3]>, Vec<[f32;2]>), Box<dyn std::error::Error>> {
    let file_extension: Option<&OsStr> = file.extension();

    return match file_extension {
        None => panic!("parse_model: path {:?} doesn't have file extension?", file),
        Some(extension) => match extension.to_str().unwrap() {
            "obj" => return obj::ObjParser::parse(file),
            _ => panic!("unsupported format (parse_image): {:?}", file)
        }
    }
}
