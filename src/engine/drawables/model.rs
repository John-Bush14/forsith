use crate::vulkan::vertex::Vertex;

use super::Drawable;

impl Drawable {
    pub fn model_from_obj(file: &str) -> Drawable {
        let mut drawable: Drawable = Default::default();

        drawable.pipeline_id = super::PIPELINE_3D;


        drawable.vertices = vec!();
        drawable.coords = vec!();

        let (mut models, _) = tobj::load_obj(std::path::Path::new(file)).unwrap();
        while let Some(model) = models.pop() {
            let mesh = model.mesh;

            let positions = mesh.positions.as_slice();
            let coords = mesh.texcoords.as_slice();
            let indices = mesh.indices.as_slice();

            drawable.vertices.reserve(indices.len());
            drawable.coords.reserve(indices.len());

            for indice in indices {
                if *indice as usize >= positions.len() {panic!("what the fuck! {:?}", indice);}

                let pi = (indice * 3) as usize;
                let ci = (indice * 2) as usize;

                let x = positions[pi];
                let y = positions[pi + 1];
                let z = positions[pi + 2];
                let u = coords[ci];
                let v = coords[ci + 1];

                drawable.vertices.push(Vertex {
                    pos: [x, y, z],
                    coord: [0.0;2],
                    color: [1.0;4]
                });

                drawable.coords.push([
                    u,
                    1.0 - v
                ]);
            }
        }


        return drawable;
    }
}
