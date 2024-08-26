use crate::vulkan::vertex::Vertex;

use super::Drawable;

impl Drawable {
    pub fn model_from_obj(file: &str) -> Drawable {
        let mut drawable: Drawable = Default::default();

        drawable.pipeline_id = super::PIPELINE_3D;


        let (models, _) = tobj::load_obj(std::path::Path::new(file)).unwrap();
        let mesh = &models[0].mesh;

        let positions = mesh.positions.as_slice();
        let coords = mesh.texcoords.as_slice();
        let indices = mesh.indices.as_slice();


        drawable.vertices = Vec::with_capacity(indices.len());
        drawable.coords = Vec::with_capacity(indices.len());


        for indice in indices {
            let pi = (indice * 3) as usize;
            let ci = (indice * 2) as usize;

            let x = positions[pi * 3];
            let y = positions[pi * 3 + 1];
            let z = positions[pi * 3 + 2];
            let u = coords[ci];
            let v = coords[ci + 1];

            drawable.vertices.push(Vertex {
                pos: [x, y, z],
                coord: [0.0;2],
                color: [1.0;4]
            });

            drawable.coords.push([
                u,
                v
            ]);
        }


        return drawable;
    }
}
