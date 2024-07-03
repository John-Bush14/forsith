use super::vulkan::abstractions::Texture;
use super::vulkan::abstractions::Vertex;

pub trait Drawable<'d> {
    type creationEnum;

    fn new(parameters: Self::creationEnum) -> &'d mut Self;

    fn get_verteces(&self) -> Vec<Vertex>;
    
    fn get_translation(&self) -> [[i32;2];2];
}

