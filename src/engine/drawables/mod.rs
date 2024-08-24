use crate::vulkan::{
    image::{Texture, VkImage, VkImageView, VkSampler}, pipeline::{BuiltinUniform, GraphicsPipeline, ShaderItem, ShaderStage, UniformType}, uniform::VkDescriptorSet, vertex::{
        Vertex,
        VkBuffer,
        VkDeviceMemory
    }
};

pub mod quads;


#[allow(unused_imports)]
pub(self) use crate::{
    PIPELINE_3D,
    PIPELINE_2D,
    PIPELINE_UI_3D,
    PIPELINE_UI_2D,
};

pub(self) use crate::engine::update_memory;


pub type Color = [f32; 4];


pub struct Drawable {
    drawing: bool,
    pos: [f32;3],
    scale: [f32;3],
    rot: f32,
    tex: Color,
    translation: [[f32;4];4],
    pub(crate) uniform_buffers: Vec<Vec<(VkBuffer, VkDeviceMemory)>>,
    pub(crate) indice_buffer: VkBuffer,
    pub(crate) indice_memory: VkDeviceMemory,
    vertices: Vec<Vertex>,
    pub(crate) descriptor_sets: Vec<VkDescriptorSet>,
    pub(crate) indices: Vec<u16>,
    pub(crate) id: usize,
    matrix_changed: bool,
    pub(crate) vertices_changed: (bool, bool),
    indices_changed: (bool, bool),
    pub(crate) device: u64,
    pipeline_id: usize,
    pub(crate) coords: Vec<[f32;2]>,
    pub(crate) uniforms: std::collections::HashMap<ShaderStage, Vec<ShaderItem>>
}


pub(self) fn points_to_vertices(points: Vec<[f32;3]>, color: Color) -> Vec<Vertex> {
    points.iter().map(|&point|
        return Vertex {pos: point, color, coord: [0.0;2]}
    ).collect()
}

pub(self) fn points_to_coords(points: Vec<[f32;3]>) -> Vec<[f32;2]> {
    return points.iter().map(|point| [point[0] + 0.5, point[1] + 0.5]).collect();
}

impl Drawable {
    pub(crate) fn get_vertices(&self) -> &Vec<Vertex> {
        return &self.vertices
    }

    pub(crate) fn update(&mut self, _image_index: usize, device: u64, pipeline: &GraphicsPipeline) -> (bool, (bool, bool), (bool, bool)) {
        let result = (self.matrix_changed, self.vertices_changed, self.indices_changed);

        let mut uniform_buffer_i = 0;

        for (stage, uniform_types) in pipeline.uniform_layout.iter() {
            let uniforms = self.uniforms.get(&stage).unwrap();

            for i in 0 .. uniform_types.len() {
                let uniform_type = &uniform_types[i];

                let _uniform = &uniforms[i];

                match uniform_type {
                    UniformType::Builtin(builtin) => {
                        match builtin {BuiltinUniform::Camera2d => continue, BuiltinUniform::Camera3d => continue, _ => {}}


                        let uniform_buffer = &self.uniform_buffers[uniform_buffer_i];

                        uniform_buffer_i += 1;


                        if !self.matrix_changed {continue}


                        match builtin {
                            BuiltinUniform::Model2d => {
                                let rot_radians = self.rot.to_radians();
                                let cos = rot_radians.cos(); let sin = rot_radians.sin();

                                self.translation = [
                                    [cos*self.scale[0], sin, 0.0, self.pos[0]],
                                   [-sin, cos*self.scale[1], 0.0, self.pos[1]],
                                    [0.0, 0.0, 0.0, 0.0],
                                    [0.0, 0.0, 0.0, 1.0]
                                ];
                            },

                            BuiltinUniform::Model3d => {
                                let rot_radians = self.rot.to_radians();
                                let cos = rot_radians.cos(); let sin = rot_radians.sin();

                                self.translation = [
                                    [cos*self.scale[0], sin, 0.0, self.pos[0]],
                                    [-sin, cos*self.scale[1], 0.0, self.pos[1]],
                                    [0.0, 0.0, self.scale[2], self.pos[2]],
                                   [0.0, 0.0, 0.0, 1.0]
                                ];
                            },

                            _ => {}
                        }


                        for image_index in 0 .. uniform_buffer.len() {
                            update_memory(uniform_buffer[image_index].1, device, self.translation);
                        }

                        self.matrix_changed = false;
                    },
                    UniformType::Local(_shader_type) => {
                        if uniform_type.size_of() == 0 {continue;}
                    },
                    _ => {}
                }
            }
        }

        return result;
    }

    pub fn is_drawing(&self) -> bool {return self.drawing}

    pub fn get_id(&self) -> usize {return self.id}

    pub fn set_pipeline_id(&mut self, pipeline_id: usize) {
        self.pipeline_id = pipeline_id;
    }
}

impl Drawable {
    pub(crate) fn update_vertice_coords(&mut self) {
        self.vertices.iter_mut().enumerate().for_each(|(i, vertice)| vertice.coord = self.coords[i]);
    }

    pub(crate) fn matrix_change(&mut self) {self.matrix_changed = true}

    pub fn pos(&self) -> &[f32;3] {return &self.pos}
    pub fn set_pos(&mut self, pos: [f32;3]) {self.pos = pos; self.matrix_change();}

    pub fn scale(&self) -> &[f32; 3] {return &self.scale}
    pub fn set_scale(&mut self, scale: [f32; 3]) {self.scale = scale; self.matrix_change();}

    pub fn rot(&self) -> &f32 {return &self.rot}
    pub fn set_rot(&mut self, rot: f32) {self.rot = rot; self.matrix_change();}

    pub fn set_drawing(&mut self, drawing: bool) {self.drawing = drawing;}

    pub fn get_pipeline_id(&self) -> usize {return self.pipeline_id}
}

impl Default for Drawable {
    fn default() -> Drawable {
        let uniforms = std::collections::HashMap::new();

        return Drawable {
            drawing: true,
            pos: [0f32, 0f32, 0f32],
            scale: [1f32; 3],
            rot: 0f32,
            tex: [0f32;4],
            translation: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            uniform_buffers: vec!(),
            descriptor_sets: vec!(),
            indice_buffer: 0,
            indice_memory: 0,
            vertices: vec!(),
            indices: vec!(),
            id: 0usize,
            matrix_changed: true,
            vertices_changed: (false, false),
            indices_changed: (true, true),
            device: 0,
            pipeline_id: PIPELINE_3D,
            coords: vec!(),
            uniforms
        };
    }
}
