use super::ModelParser;
use std::path::Path;
use crate::vulkan::vertex::Vertex;


pub struct ObjParser {}


impl ModelParser for ObjParser {
    fn parse(_path: &Path) -> Result<Vec<Vec<Vertex>>, Box<(dyn std::error::Error + 'static)>> {todo!();}
}
