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
    matrix_changed: bool,
    pub vertices_changed: (bool, bool),
    indices_changed: (bool, bool),
    pub device: u64
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

    pub fn update(&mut self, image_index: usize, aspect: f32, device: u64) -> (bool, (bool, bool), (bool, bool)) { 
        let result = (self.matrix_changed, self.vertices_changed, self.indices_changed);
        

        if true {
            let rot_radians = self.rot.to_radians();
            let cos = rot_radians.cos(); let sin = rot_radians.sin();

            self.translation = [
                [cos*self.scale[0], sin, 0.0, self.pos[0]],
               [-sin, cos*self.scale[1], 0.0, self.pos[1]],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ];


            println!("{:?}", self.translation);

            update_uniform_buffer(self.uniform_memories[image_index], self.translation, aspect, device);

            self.matrix_changed = false;
        }

        return result;
    }
    
    pub fn get_texture(&self) -> &Texture {return &self.tex}

    pub fn is_drawing(&self) -> bool {return self.drawing}

    pub fn get_id(&self) -> usize {return self.id}
}

impl drawable {
    pub fn pos(&self) -> &[f32;2] {return &self.pos}
    pub fn set_pos(&mut self, pos: [f32;2]) {self.pos = pos; self.matrix_changed = true;}

    pub fn scale(&self) -> &[f32; 2] {return &self.scale}
    pub fn set_scale(&mut self, scale: [f32; 2]) {self.scale = scale; self.matrix_changed = true;}
    
    pub fn rot(&self) -> &f32 {return &self.rot}
    pub fn set_rot(&mut self, rot: f32) {self.rot = rot; self.matrix_changed = true;}

    pub fn set_texture(&mut self, texture: Texture) {self.tex = texture;}

    pub fn set_drawing(&mut self, drawing: bool) {self.drawing = drawing;}
}

impl drawable {
    pub fn new() -> drawable {
        return drawable {
            drawing: true,
            pos: [0f32;2],
            scale: [1f32; 2],
            rot: 0f32,
            tex: [0f32;4],
            translation: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            uniform_buffers: vec!(),
            uniform_memories: vec!(),
            descriptor_sets: vec!(),
            indice_buffer: 0,
            indice_memory: 0,
            vertices: points_to_vertices(vec!([-0.5, -0.5], [0.5, -0.5], [0.5, 0.5]), [0.5, 1.0, 0.4, 1.0]),
            indices: vec!(),
            id: 0usize,
            matrix_changed: true,
            vertices_changed: (false, false),
            indices_changed: (true, true),
            device: 0
        };
    }

    pub fn special(parameters: DrawableCreation) -> drawable {
        let mut drawable = drawable::new();

        match parameters {
            DrawableCreation::Line2DFromPoints(p1, p2, col) => {
                drawable.tex = col;
                drawable.pos = [(p1[0] + p2[0]) / 2.0, (p1[1] + p2[1]) /2.0];
                drawable.scale = [((p2[0] - p1[0]).powi(2) + (p2[1] - p1[1]).powi(2)).sqrt(), 0.0];
                drawable.rot = p1[1].atan2(p1[0]).to_degrees();
                drawable.vertices = points_to_vertices(RECT2D_POINTS.to_vec(), col);
            },
            
            DrawableCreation::Rect2DFromTransform(pos, scale, rot, col) => {
                drawable.tex = col;
                drawable.pos = pos;
                drawable.scale = scale;
                drawable.rot = rot;
                drawable.vertices = points_to_vertices(RECT2D_POINTS.to_vec(), col);
            },
        };

        return drawable;
    }
}

pub fn update_uniform_buffer(buffer_memory: VkDeviceMemory, model: [[f32;4];4], aspect: f32, device: u64) {
    let ubo = UniformBufferObject {
        model: model,
        view: {
            let eye = [2.0, 2.0, 2.0];
            let target = [0.0, 0.0, 0.0];
            let up = [0.0, 0.0, 1.0];

            let zaxis = {
                let x = [eye[0] - target[0], eye[1], target[1], eye[2], target[2]];

                let l = x.len() as f32;

                [x[0]/l, x[1]/l, x[2]/l]
            };

            let xaxis = {
                let x = [
                    up[1] * zaxis[2] - up[2] * zaxis[1],
                    up[2] * zaxis[0] - up[0] * zaxis[2],
                    up[0] * zaxis[1] - up[1] * zaxis[0]
                ];

                let l = x.len() as f32;

                [x[0]/l, x[1]/l, x[2]/l]
            };

            let yaxis = {
                [
                    zaxis[1] * xaxis[2] - xaxis[2] * zaxis[1],
                    zaxis[2] * xaxis[0] - zaxis[0] * xaxis[2],
                    zaxis[0] * xaxis[1] - zaxis[1] * xaxis[0]
                 ]
            };

            [[xaxis[0], yaxis[0], zaxis[0], 0f32],
             [xaxis[1], yaxis[1], zaxis[1], 0f32],
             [xaxis[2], yaxis[2], zaxis[2], 0f32],
             [-dot(xaxis, eye), -dot(yaxis, eye), -dot(zaxis, eye), 1.0]];
             
                 [[1.0, 0.0, 0.0, 0.0], 
              [0.0, 1.0, 0.0, 0.0], 
              [0.0, 0.0, 1.0, 0.0], 
              [0.0, 0.0, 0.0, 1.0]];

            Matrix4::look_at(
                Point3::new(2.0, 2.0, 2.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
            ).into()
        },
        proj: {
            let f = 1.0/(45f32.to_radians()/2.0).tan();
            
            let near = 0.1;
            let far = 10.0;

            [[f/aspect, 0.0, 0.0, 0.0],
             [0.0, -f, 0.0, 0.0],
             [0.0, 0.0, -far/(far-near), -1.0],
             [0.0, 0.0, -(far * near) / (far - near), 0.0]]
        } 
    };

    let size = std::mem::size_of::<UniformBufferObject>() as u64;

    let mut data_ptr: *mut std::ffi::c_void = std::ptr::null_mut();

    unsafe {vkMapMemory(
        device,
        buffer_memory,
        0,
        size,
        0,
        &mut data_ptr as _
    )};

    let align = std::mem::align_of::<f32>();

    let layout = std::alloc::Layout::from_size_align(size as usize, align).unwrap();
    
    let ubos = [ubo];

    unsafe {std::ptr::copy_nonoverlapping(ubos.as_ptr(), data_ptr as _, ubos.len())};

    unsafe {vkUnmapMemory(device, buffer_memory)};
}

fn dot(x: [f32;3], y: [f32;3]) -> f32 {
    return (x[0] * y[0]) + (x[1] * y[1]) + (x[2] * y[2]);
}

impl Drop for drawable {
    fn drop(&mut self) { unsafe {
        self.uniform_buffers.iter().zip(self.uniform_memories.iter()).for_each(|(&buffer, &memory)| {
            vkDestroyBuffer(self.device, buffer, std::ptr::null());
            vkFreeMemory(self.device, memory, std::ptr::null());
        });
        
        vkDestroyBuffer(self.device, self.indice_buffer, std::ptr::null());
        vkFreeMemory(self.device, self.indice_memory, std::ptr::null());
    }}
}
