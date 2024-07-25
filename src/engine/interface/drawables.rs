use crate::vulkan::{
    vertex::{
        Vertex
    }
};


pub type Texture = [f32; 4];


pub struct drawable {
    drawing: bool,
    pos: [f32;2],
    scale: [f32; 2],
    rot: f32,
    tex: Texture,
    translation: [[f32;4];4],
    vertices: Vec<Vertex>,
    pub indices: Vec<usize>,
    recalculate: bool,
    pub id: usize,
}


const RECT2D_POINTS: [[f32; 2]; 6] = [
    [-0.5, -0.5],
    [0.5, -0.5],
    [0.5, 0.5],
    [0.5, 0.5],
    [-0.5, 0.5],
    [-0.5, -0.5]
];


fn points_to_vertices(points: Vec<[f32;2]>, color: Texture) -> Vec<Vertex> {
    points.iter().map(|&point| return Vertex {pos: point, color: color}).collect()
}


pub enum DrawableCreation {
    Line2DFromPoints([f32;2], [f32;2], Texture),
    Rect2DFromTransform([f32;2], [f32; 2], f32, Texture)
}   

impl drawable {
    pub fn get_vertices(&self) -> &Vec<Vertex> {
        return &self.vertices
    }

    pub fn get_translation(&mut self) -> &[[f32;4];4] { 
        if self.recalculate {
            let rot_radians = self.rot.to_radians();
            let cos = rot_radians.cos(); let sin = rot_radians.sin();

            self.translation = [
                [cos*self.scale[0], sin, 0.0, self.pos[0]],
               [-sin, cos*self.scale[1], 0.0, self.pos[1]],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ];

            self.recalculate = false;
        }

        return &self.translation
    }
    
    pub fn get_texture(&self) -> &Texture {return &self.tex}

    pub fn is_drawing(&self) -> bool {return self.drawing}

    pub fn get_id(&self) -> usize {return self.id}
}

impl drawable {
    pub fn pos(&self) -> &[f32;2] {return &self.pos}
    pub fn set_pos(&mut self, pos: [f32;2]) {self.pos = pos; self.recalculate = true;}

    pub fn scale(&self) -> &[f32; 2] {return &self.scale}
    pub fn set_scale(&mut self, scale: [f32; 2]) {self.scale = scale; self.recalculate = true;}
    
    pub fn rot(&self) -> &f32 {return &self.rot}
    pub fn set_rot(&mut self, rot: f32) {self.rot = rot; self.recalculate = true;}

    pub fn set_texture(&mut self, texture: Texture) {self.tex = texture;}

    pub fn set_drawing(&mut self, drawing: bool) {self.drawing = drawing;}
}

impl drawable {
    fn new(parameters: DrawableCreation) -> drawable {
        return match parameters {
            DrawableCreation::Line2DFromPoints(p1, p2, col) => drawable {
                drawing: true,
                pos: [(p1[0] + p2[0]) / 2.0, (p1[1] + p2[1]) /2.0],
                scale: [((p2[0] - p1[0]).powi(2) + (p2[1] - p1[1]).powi(2)).sqrt(), 0.0],
                rot: p1[1].atan2(p1[0]).to_degrees(),
                tex: col,               
                translation: [[0f32;4];4],
                recalculate: true, 
                vertices: points_to_vertices(RECT2D_POINTS.to_vec(), col),
                indices: vec!(),
                id: 0
            },
            
            DrawableCreation::Rect2DFromTransform(pos, scale, rot, col) => drawable {
                drawing: true, 
                pos: pos,
                scale: scale, 
                rot: rot,
                tex: col,
                translation: [[0f32;4];4],
                recalculate: true,
                vertices: points_to_vertices(RECT2D_POINTS.to_vec(), col),
                indices: vec!(),
                id: 0
            },
        };
    }
}
