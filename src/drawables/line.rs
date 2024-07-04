use super::Vertex;
use super::Texture;
use super::newDrawable;
use super::super::globals::STATE;

pub struct Line {
    drawing: bool,
    pos: Vertex,
    len: f32,
    rot: f32,
    tex: Texture,
    translation: [[f32;3];3],
    recalculate: bool,
    id: usize,
}

pub enum NewLine {
    Points(Vertex, Vertex, Texture),
    Transform(Vertex, f32, f32, Texture)
}   

impl super::Drawable for Line {
    fn new(parameters: newDrawable) -> usize { match parameters { newDrawable::Line(parameters) => {
        let mut drawables_len = 0usize;
        unsafe {drawables_len = STATE.drawables.len();};

        let mut line = match parameters {
            NewLine::Points(p1, p2, tex) => Line {
                drawing: true,
                pos: Vertex {x: (p1.x + p2.x) / 2.0, y: (p1.y + p2.y) /2.0},
                len: ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt(),
                rot: p1.y.atan2(p1.x).to_degrees(),
                tex: tex,               
                translation: [[0f32;3];3],
                recalculate: true, 
                id: drawables_len
            },
            
            NewLine::Transform(pos, len, rot, tex) => Line {
                drawing: true, 
                pos: pos,
                len: len, 
                rot: rot,
                tex: tex,
                translation: [[0f32;3];3],
                recalculate: true, 
                id: drawables_len
            },
        };
        
        unsafe {
            STATE.drawables.push(Box::new(line));
            return STATE.drawables[drawables_len].get_id();
        }; 
    }, _ => {panic!("");}}}

    fn get_verteces(&self) -> Vec<Vertex> {
        vec![
            Vertex {x: 0.0, y: -1.0}, Vertex {x:0.0, y:1.0}
        ]
    }

    fn get_translation(&mut self) -> &[[f32;3];3] { 
        if self.recalculate {
            let rot_radians = self.rot.to_radians();
            let cos = rot_radians.cos(); let sin = rot_radians.sin();

            self.translation = [
                [cos, -sin, self.pos.x],
                [sin * self.len, cos * self.len, self.pos.y],
                [0.0, 0.0, 1.0]
            ];

            self.recalculate = false;
        }

        return &self.translation
    }
    
    fn get_texture(&self) -> &Texture {return &self.tex}

    fn is_drawing(&self) -> bool {return self.drawing}

    fn get_id(&self) -> usize {return self.id}
}

use crate::impl_default_setgets_drawable;

impl_default_setgets_drawable!(Line);

impl Line {
    fn set_translation(&mut self, translation: [[f32;3];3]) {
        self.rot = translation[0][0].acos();

        self.pos.x = translation[0][2];
        self.pos.y = translation[1][2];
        
        self.len = translation[1][1]/translation[0][0]
    }
}
