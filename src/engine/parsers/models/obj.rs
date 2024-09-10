use crate::vulkan::vertex::Vertex;

use super::ModelParser;
use super::Mesh;
use std::path::Path;


pub struct ObjParser {}


pub enum LineAction {
    Comment,
    NamedObject(String),
    GeometricVertice(f32, f32, f32, Option<f32>),
    TextureCoordinate(f32, Option<f32>, Option<f32>),
    Indices(Vec<(u32, Option<u32>, Option<u32>)>),
    VertexNormal(f32, f32, f32)
}


impl ModelParser for ObjParser {
    fn parse(file: &Path) -> Result<(Vec<Mesh>, Vec<[f32;3]>, Vec<[f32;2]>), Box<(dyn std::error::Error + 'static)>> {
        let file_content = std::fs::read_to_string(file)?;


        let mut meshes = vec!(Mesh::new("Main".to_string()));
        let mut vertices: Vec<[f32;3]> = vec!();
        let mut texcoords: Vec<[f32;2]> = vec!();


        file_content
            .lines().into_iter()
            .map(|file_line| Self::parse_line_action(file_line))
            .for_each(|line_action| Self::run_line_action(&mut meshes, &mut vertices, &mut texcoords, line_action));


        return Ok((meshes, vertices, texcoords));
    }
}

impl ObjParser {
    fn parse_line_action(line: &str) -> LineAction {
        let mut arguments = line.split(" ");

        return match arguments.next().unwrap() {
            "o" => LineAction::NamedObject(arguments.next().unwrap().to_string()),
            "v" => {
                let coordinates: (f32, f32, f32, Option<f32>) = (
                    arguments.next().unwrap().parse::<f32>().unwrap(),
                    arguments.next().unwrap().parse::<f32>().unwrap(),
                    arguments.next().unwrap().parse::<f32>().unwrap(),
                    if let Some(argument) = arguments.next() {Some(argument.parse::<f32>().unwrap())} else {None}
                );

                let (x, y, z, w) = coordinates;
                LineAction::GeometricVertice(x, y, z, w)
            },
            "vt" => {
                let coordinates: (f32, Option<f32>, Option<f32>) = (
                    arguments.next().unwrap().parse::<f32>().unwrap(),
                    if let Some(argument) = arguments.next() {Some(argument.parse::<f32>().unwrap())} else {None},
                    if let Some(argument) = arguments.next() {Some(argument.parse::<f32>().unwrap())} else {None}
                );

                let (u, v, w) = coordinates;
                LineAction::TextureCoordinate(u, v, w)
            },
            "vn" => {
                let coordinates: (f32, f32, f32) = (
                    arguments.next().unwrap().parse::<f32>().unwrap(),
                    arguments.next().unwrap().parse::<f32>().unwrap(),
                    arguments.next().unwrap().parse::<f32>().unwrap(),
                );

                let (x, y, z) = coordinates;
                LineAction::VertexNormal(x, y, z)
            },
            "f" => {
                let indices = arguments.map(|argument| {
                    let mut indices = argument.split("/");

                    return (
                        indices.next().unwrap().parse::<u32>().unwrap(),
                        if let Some(txcoord_indice) = indices.next() {Some(txcoord_indice.parse::<u32>().unwrap())} else {None},
                        if let Some(normal_indice) = indices.next() {Some(normal_indice.parse::<u32>().unwrap())} else {None},
                    );
                }).collect();

                LineAction::Indices(indices)
            },
            "#" => LineAction::Comment,
            line_action => {dbg!("unsupported or incorrect line action '{:?}'", line_action); LineAction::Comment}
        }
    }
}

impl ObjParser {
    fn run_line_action(meshes: &mut Vec<Mesh>, vertices: &mut Vec<[f32;3]>, texcoords: &mut Vec<[f32;2]>, line_action: LineAction) {
        let len = meshes.len();

        let mesh = &mut meshes[len-1];

        match line_action {
            LineAction::NamedObject(name) => {
                meshes.push(Mesh::new(name));
            },

            LineAction::GeometricVertice(x, y, z, _w) => {
                vertices.push([x, y, z]);
            },

            LineAction::TextureCoordinate(u, v_opt, _w) => {
                let v = if let Some(v) = v_opt {v} else {0.0};

                texcoords.push([u, v]);
            },

            LineAction::VertexNormal(_, _, _) => {},

            LineAction::Indices(indices) => {
                for indice in indices {
                    mesh.vertex_indices.push(indice.0 - 1);

                    if let Some(texcoord_indice) = indice.2 {mesh.texcoord_indices.push(texcoord_indice - 1);}
                }
            }

            LineAction::Comment => {},
        }
    }
}
