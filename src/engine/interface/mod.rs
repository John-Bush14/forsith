pub mod drawables;

use crate::vulkan::{
    vertex::{
        Vertex,
        VkBuffer,
        VkDeviceMemory,
        vkMapMemory,
        vkUnmapMemory
    },
    uniform::{
        UniformBufferObject
    }
};

use drawables::{
    drawable
};


impl crate::engine::Engine { pub fn add_drawable<'a>(&'a mut self, mut drawable: drawable) -> &'a mut drawable {
    (drawable.uniform_buffers, drawable.uniform_memories) = self.create_uniform_buffers();
    drawable.descriptor_sets = self.create_descriptor_sets(drawable.uniform_buffers.clone());

    drawable.device = self.device;

    for vertex in drawable.get_vertices().clone() {
        drawable.vertices_changed.0 = true;

        if !self.vertex_indices.contains_key(&vertex) {
            self.vertex_usage_counts.insert(vertex, 0);
        
            self.vertices.push(vertex);

            drawable.vertices_changed.1 = true;

            self.vertex_indices.insert(vertex, (self.vertices.len()-1) as u16);
        }
    
        *self.vertex_usage_counts.get_mut(&vertex).unwrap() += 1;

        drawable.indices.push(*self.vertex_indices.get(&vertex).unwrap());
    }

    if drawable.indices.len() > 0 {
        (drawable.indice_buffer, drawable.indice_memory)
            = self.create_device_local_buffer_with_data::<u16, _>(0x00000040, &drawable.indices);
    }
    
    drawable.id = self.drawables.len() + 1;

    let id = self.drawables.len();

    self.drawables.push(drawable);

    return &mut self.drawables[id];
}}

impl crate::engine::Engine { pub fn remove_drawable(&mut self, drawable_index: usize) {
    let drawable = self.drawables.remove(drawable_index);

    for i in 0..drawable.get_vertices().len() {
        let vertex = drawable.get_vertices()[i];

        
        let usage_count = self.vertex_usage_counts.get_mut(&vertex).unwrap();

        *usage_count -= 1;


        if *usage_count <= 0 {
            let indice = *self.vertex_indices.get(&vertex).unwrap();

            self.vertices.remove(indice as usize);
            self.vertex_indices.remove(&vertex);

            for vertex in &self.vertices {
                let vertex_indice = self.vertex_indices.get_mut(&vertex).unwrap();
                
                if *vertex_indice >= indice {*vertex_indice -= 1;}
            }
            
            for drawable in &mut self.drawables {for indice_t in &mut drawable.indices {if *indice_t >= indice as u16 {*indice_t -= 1}}}
        }
    }
}}

impl crate::engine::Engine { pub fn create_uniform_buffers(&self) -> (Vec<VkBuffer>, Vec<VkDeviceMemory>) {
    let size = std::mem::size_of::<UniformBufferObject>() as u64;

    let mut uniform_buffers = vec!();
    let mut uniform_memories = vec!();

    for _ in 0 .. self.swapchain_images.len() {
        let (uniform_buffer, uniform_buffer_memory, _) = 
            self.create_buffer(
                size,
                0x00000010,
                0x00000002 | 0x00000004
        );

        uniform_buffers.push(uniform_buffer);
        uniform_memories.push(uniform_buffer_memory);
    }

    return (uniform_buffers, uniform_memories);
}}
