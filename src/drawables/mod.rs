pub use super::vulkan::abstractions::Texture;
pub use super::vulkan::abstractions::Vertex;

#[macro_export]
macro_rules! impl_default_setgets_drawable {
    ($drawable:ty) => {
        impl $drawable {
            fn pos(&self) -> &Vertex {return &self.pos}
            fn set_pos(&mut self, pos: Vertex) {self.pos = pos; self.recalculate = true;}

            fn len(&self) -> &f32 {return &self.len}
            fn set_len(&mut self, len: f32) {self.len = len; self.recalculate = true;}
    
            fn rot(&self) -> &f32 {return &self.len}
            fn set_rot(&mut self, rot: f32) {self.rot = rot; self.recalculate = true;}

            fn set_texture(&mut self, texture: Texture) {self.tex = texture;}

            fn set_drawing(&mut self, drawing: bool) {self.drawing = drawing;}
        }
    };
}

mod line; use line::NewLine;

pub enum newDrawable {
    Line(NewLine)
}

pub trait Drawable {
    fn new(parameters: newDrawable) -> usize where Self: Sized;

    fn get_verteces(&self) -> Vec<Vertex>;
    
    fn get_translation(&mut self) -> &[[f32;3];3];

    fn get_texture(&self) -> &Texture;

    fn is_drawing(&self) -> bool;

    fn get_id(&self) -> usize;
}

pub use line::Line;
