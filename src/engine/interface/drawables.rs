use crate::vulkan::{
    vertex::{
        Vertex,
        VkBuffer,
        VkDeviceMemory,
        vkMapMemory,
        vkFreeMemory,
        vkUnmapMemory,
        vkDestroyBuffer
    },
    uniform::{
        VkDescriptorSet,
        UniformBufferObject
    }
};

use crate::engine::{
    world_view::worldView,
    initialisation::buffer::update_uniform_buffer
};

use cgmath::{Deg, Matrix4, Point3, Vector3};


pub type Texture = [f32; 4];


pub struct drawable {
    drawing: bool,
    pos: [f32;2],
    scale: [f32; 2],
    rot: f32,
    tex: Texture,
    translation: [[f32;4];4],
    pub uniform_buffers: Vec<VkBuffer>,
    pub uniform_memories: Vec<VkDeviceMemory>,
    pub indice_buffer: VkBuffer,
    pub indice_memory: VkDeviceMemory,
    vertices: Vec<Vertex>,
    pub descriptor_sets: Vec<VkDescriptorSet>,
    pub indices: Vec<u16>,
    pub id: usize,
    matrix_changed: u8,
    pub vertices_changed: (bool, bool),
    indices_changed: (bool, bool),
    pub device: u64,
    pub two_d: bool
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


impl drawable {
    pub fn get_vertices(&self) -> &Vec<Vertex> {
        return &self.vertices
    }

    pub fn update(&mut self, image_index: usize, aspect: f32, device: u64, world_view: &mut worldView) -> (bool, (bool, bool), (bool, bool)) { 
        let result = (self.matrix_changed != 0, self.vertices_changed, self.indices_changed);

        if world_view.changed.0 || world_view.changed.1 || world_view.aspect != aspect {self.matrix_change()}

        if self.matrix_changed > 0 {
            let rot_radians = self.rot.to_radians();
            let cos = rot_radians.cos(); let sin = rot_radians.sin();
            
            self.translation = [
                [cos*self.scale[0], sin, 0.0, self.pos[0]],
               [-sin, cos*self.scale[1], 0.0, self.pos[1]],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ];


            self.matrix_changed -= 1;
            
            update_uniform_buffer(self.uniform_memories[image_index], self.translation, aspect, device, world_view, self.two_d);
        }

        return result;
    }
    
    pub fn get_texture(&self) -> &Texture {return &self.tex}

    pub fn is_drawing(&self) -> bool {return self.drawing}

    pub fn get_id(&self) -> usize {return self.id}
}

impl drawable {
    pub fn matrix_change(&mut self) {self.matrix_changed = self.uniform_buffers.len() as u8}

    pub fn pos(&self) -> &[f32;2] {return &self.pos}
    pub fn set_pos(&mut self, pos: [f32;2]) {self.pos = pos; self.matrix_change();}

    pub fn scale(&self) -> &[f32; 2] {return &self.scale}
    pub fn set_scale(&mut self, scale: [f32; 2]) {self.scale = scale; self.matrix_change();}
    
    pub fn set_two_d(&mut self, two_d: bool) {self.two_d = two_d; self.matrix_change()}
    
    pub fn rot(&self) -> &f32 {return &self.rot}
    pub fn set_rot(&mut self, rot: f32) {self.rot = rot; self.matrix_change();}

    pub fn set_texture(&mut self, texture: Texture) {self.tex = texture;}

    pub fn set_drawing(&mut self, drawing: bool) {self.drawing = drawing;}
}

impl Default for drawable {
    fn default() -> drawable {
        return drawable {
            drawing: true,
            pos: [0f32, 0f32],
            scale: [1f32; 2],
            rot: 0f32,
            tex: [0f32;4],
            translation: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            uniform_buffers: vec!(),
            uniform_memories: vec!(),
            descriptor_sets: vec!(),
            indice_buffer: 0,
            indice_memory: 0,
            vertices: points_to_vertices(vec!(
                    [-0.5, -0.5],
                    [0.5, -0.5],
                    [0.5, 0.5],
                    [0.5, 0.5],
                    [-0.5, 0.5],
                    [-0.5, -0.5],

                    [-0.5, -0.5],
                    [-0.5, 0.5],
                    [0.5, 0.5],
                    [0.5, 0.5],
                    [0.5, -0.5],
                    [-0.5, -0.5]
                ), [0.5, 1.0, 0.4, 1.0]),
            indices: vec!(),
            id: 0usize,
            matrix_changed: 0,
            vertices_changed: (false, false),
            indices_changed: (true, true),
            device: 0,
            two_d: true
        };
    }
}

impl drawable {
    pub fn line2d_from_points(p1: [f32; 2], p2: [f32; 2], col: Texture) -> drawable {
        let mut drawable: drawable = Default::default();

        drawable.tex = col;
        drawable.pos = [(p1[0] + p2[0]) / 2.0, (p1[1] + p2[1]) /2.0];
        drawable.scale = [((p2[0] - p1[0]).powi(2) + (p2[1] - p1[1]).powi(2)).sqrt(), 0.0];
        drawable.rot = p1[1].atan2(p1[0]).to_degrees();
        drawable.vertices = points_to_vertices(RECT2D_POINTS.to_vec(), col);

        return drawable;
    }
    
    pub fn rect2D_from_transform(pos: [f32;2], width: f32, height: f32, rot: f32, col: Texture) -> drawable {
        let mut drawable: drawable = Default::default();

        drawable.tex = col;
        drawable.pos = pos;
        drawable.scale = [width, height];
        drawable.rot = rot;
        drawable.vertices = points_to_vertices(RECT2D_POINTS.to_vec(), col);

        return drawable;
    }
}
