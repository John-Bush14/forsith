use super::ModelParser;
use std::path::Path;
use crate::vulkan::vertex::Vertex;


#[derive(Default)]
pub struct ObjParser {
    meshes: Vec<Vec<Vertex>>
}

pub enum LineAction {
    Comment
}


impl ModelParser for ObjParser {
    fn parse(file: &Path) -> Result<Vec<Vec<Vertex>>, Box<(dyn std::error::Error + 'static)>> {
        let file_content = std::fs::read_to_string(file)?;


        let mut parser: ObjParser = Default::default();


        file_content
            .lines().into_iter()
            .map(|file_line| Self::parse_line_action(file_line))
            .for_each(|line_action| parser.run_line_action(line_action));


        return Ok(parser.meshes);
    }
}

impl ObjParser {
    fn parse_line_action(line: &str) -> LineAction {
        let mut arguments = line.split(" ");

        return match arguments.next().unwrap() {
            "o" => todo!(),
            "v" => todo!(),
            "vt" => todo!(),
            "vn" => todo!(),
            "f" => todo!(),
            "#" => LineAction::Comment,
            line_action => panic!("unsupported or incorrect line action '{:?}'", line_action)
        }
    }
}

impl ObjParser {
    fn run_line_action(&mut self, line_action: LineAction) {todo!();}
}
