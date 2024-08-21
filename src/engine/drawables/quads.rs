use super::{points_to_vertices, Drawable, Color};


const RECT: [[f32; 3]; 6] = [
    [-0.5, -0.5, 0.0],
    [-0.5, 0.5, 0.0],
    [0.5, 0.5, 0.0],
    [0.5, 0.5, 0.0],
    [0.5, -0.5, 0.0],
    [-0.5, -0.5, 0.0],
];

const RECT_COORDS: [[f32; 2]; 6] = [
    [0.0, 0.0],
    [0.0, 1.0],
    [1.0, 1.0],
    [1.0, 1.0],
    [1.0, 0.0],
    [0.0, 0.0],
];

const CUBE: [[f32; 3]; 36] = [
    // Front face
    [ -0.5, -0.5,  0.5 ], // Bottom-left
    [  0.5, -0.5,  0.5 ], // Bottom-right
    [  0.5,  0.5,  0.5 ], // Top-right
    [ -0.5, -0.5,  0.5 ], // Bottom-left
    [  0.5,  0.5,  0.5 ], // Top-right
    [ -0.5,  0.5,  0.5 ], // Top-left

    // Back face
    [ -0.5,  0.5, -0.5 ], // Bottom-left
    [  0.5,  0.5, -0.5 ], // Bottom-right
    [ -0.5, -0.5, -0.5 ], // Top-right
    [  0.5,  0.5, -0.5 ], // Bottom-left
    [  0.5, -0.5, -0.5 ], // Top-right
    [ -0.5, -0.5, -0.5 ], // Top-left

    // Left face
    [ -0.5, -0.5, -0.5 ], // Bottom-left
    [ -0.5, -0.5,  0.5 ], // Bottom-right
    [ -0.5,  0.5,  0.5 ], // Top-right
    [ -0.5, -0.5, -0.5 ], // Bottom-left
    [ -0.5,  0.5,  0.5 ], // Top-right
    [ -0.5,  0.5, -0.5 ], // Top-left

    // Right face
    [  0.5,  0.5, -0.5 ], // Bottom-left
    [  0.5,  0.5,  0.5 ], // Bottom-right
    [  0.5, -0.5, -0.5 ], // Top-right
    [  0.5,  0.5,  0.5 ], // Bottom-left
    [  0.5, -0.5,  0.5 ], // Top-right
    [  0.5, -0.5, -0.5 ], // Top-left

    // Top face
    [ -0.5,  0.5,  0.5 ], // Bottom-left
    [  0.5,  0.5,  0.5 ], // Bottom-right
    [ -0.5,  0.5, -0.5 ], // Top-right
    [  0.5,  0.5,  0.5 ], // Bottom-left
    [  0.5,  0.5, -0.5 ], // Top-right
    [ -0.5,  0.5, -0.5 ], // Top-left

    // Bottom face
    [ -0.5, -0.5, -0.5 ], // Bottom-left
    [  0.5, -0.5, -0.5 ], // Bottom-right
    [  0.5, -0.5,  0.5 ], // Top-right
    [ -0.5, -0.5, -0.5 ], // Bottom-left
    [  0.5, -0.5,  0.5 ], // Top-right
    [ -0.5, -0.5,  0.5 ], // Top-left
];

impl Drawable {
    pub fn cube_from_transform(pos: [f32;3], width: f32, height: f32, depth: f32, col: Color) -> Drawable {
        let mut drawable: Drawable = Default::default();

        drawable.tex = col;
        drawable.pos = pos;
        drawable.scale = [width, height, depth];
        drawable.vertices = points_to_vertices(CUBE.to_vec(), None, col);

        return drawable;
    }
    
    pub fn rect_from_transform(pos: [f32;2], width: f32, height: f32, rot: f32, col: Color, ui: bool, image: bool) -> Drawable {
        let mut drawable: Drawable = Default::default();

        drawable.tex = col;
        drawable.pos = [pos[0], pos[1], 0.0];
        drawable.scale = [width, height, 1.0];
        drawable.rot = rot;
        drawable.vertices = points_to_vertices(RECT.to_vec(), Some(RECT_COORDS.to_vec()), col);
        drawable.pipeline_id = if ui {super::PIPELINE_UI_2D} else if image {4} else {super::PIPELINE_2D};

        return drawable;
    }
}
