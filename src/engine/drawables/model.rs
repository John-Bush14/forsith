use crate::vulkan::vertex::Vertex;
use super::Drawable;
use crate::engine::parsers::parse_model;


impl Drawable {
    pub fn model_from_obj(file: &str) -> Drawable {
        let mut drawable: Drawable = Default::default();

        drawable.pipeline_id = super::PIPELINE_3D;


        drawable.vertices = vec!();
        drawable.coords = vec!();


        let mut meshes = parse_model(&std::path::Path::new(file)).unwrap();
        while let Some(mesh) = meshes.pop() {

            let vertices = mesh.vertices.as_slice();
            let texcoords = mesh.texcoords.as_slice();

            let vertex_indices = mesh.vertex_indices.as_slice();
            let texcoord_indices = mesh.texcoord_indices.as_slice();

            drawable.vertices.reserve(vertex_indices.len());
            drawable.coords.reserve(texcoord_indices.len());

            for indice in vertex_indices {
                let pos = vertices[*indice as usize];

                drawable.vertices.push(Vertex {
                    pos,
                    coord: [0.0;2],
                    color: [1.0;4]
                });
            }

            for indice in texcoord_indices {
                let [u, v] = texcoords[*indice as usize];

                drawable.coords.push([
                    u,
                    1.0 - v
                ]);
            }
        }


        return drawable;
    }
}
