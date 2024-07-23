use crate::vulkan::{
    uniform::{
        UniformBufferObject,
        VkDescriptorPoolSize,
        VkWriteDescriptorSet,
        VkDescriptorBufferInfo,
        VkDescriptorPoolCreateInfo,
        VkDescriptorSetAllocateInfo,
        VkDescriptorSetLayoutBinding,
        VkDescriptorSetLayoutCreateInfo,
        vkUpdateDescriptorSets,
        vkCreateDescriptorPool,
        vkAllocateDescriptorSets,
        vkCreateDescriptorSetLayout
    },
    vertex::{
        vkMapMemory,
        vkUnmapMemory
    }
};

use cgmath::{Deg, Matrix4, Point3, Vector3};

impl crate::engine::Engine { pub fn create_descriptor_sets(&mut self) {
    let layouts: Vec<_> = (self.uniform_buffers).iter().map(|_| self.descriptor_set_layout).collect();

    let allocate_info = VkDescriptorSetAllocateInfo {
        s_type: 34,
        p_next: std::ptr::null(),
        descriptor_pool: self.descriptor_pool,
        descriptor_set_count: self.uniform_buffers.len() as u32,
        set_layouts: layouts.as_ptr()
    };

    self.descriptor_sets = (self.uniform_buffers).iter().map(|_| 0).collect();

    unsafe {vkAllocateDescriptorSets(self.device, &allocate_info as *const VkDescriptorSetAllocateInfo, self.descriptor_sets.as_mut_ptr())};

    
    self.descriptor_sets.iter().zip(self.uniform_buffers.iter()).for_each(|(&set, &buffer)| {
        let buffer_info = VkDescriptorBufferInfo {
            buffer: buffer,
            offset: 0,
            range: std::mem::size_of::<UniformBufferObject>() as u64
        }; let buffer_infos = [buffer_info];
        
        
        let descriptor_write = VkWriteDescriptorSet {
            s_type: 35,
            p_next: std::ptr::null(),
            dst_set: set,
            dst_binding: 0,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: 6,
            image_info: std::ptr::null(),
            buffer_info: buffer_infos.as_ptr(),
            texel_buffer_view: std::ptr::null()
        }; let descriptor_writes = [descriptor_write];


        unsafe {vkUpdateDescriptorSets(
            self.device,
            descriptor_writes.len() as u32,
            descriptor_writes.as_ptr(),
            0,
            std::ptr::null()
        )};
    }); 
}}

impl crate::engine::Engine { pub fn create_descriptor_set_layout(&mut self) {
    let binding = VkDescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: 6,
        descriptor_count: 1,
        stage_flags: 0x00000001,
        immutable_samplers: std::ptr::null()
    };

    let bindings = [binding];

    let create_info = VkDescriptorSetLayoutCreateInfo {
        s_type: 32,
        p_next: std::ptr::null(),
        flags: 0,
        binding_count: bindings.len() as u32,
        bindings: bindings.as_ptr()
    };

    unsafe {vkCreateDescriptorSetLayout(
        self.device, 
        &create_info as *const VkDescriptorSetLayoutCreateInfo, 
        std::ptr::null(), 
        &mut self.descriptor_set_layout
    )};
}}

impl crate::engine::Engine { pub fn create_uniform_buffers(&mut self) {
    let size = std::mem::size_of::<UniformBufferObject>() as u64;

    for _ in 0 .. self.swapchain_images.len() {
        let (uniform_buffer, uniform_buffer_memory, _) = 
            self.create_buffer(
                size,
                0x00000010,
                0x00000002 | 0x00000004
        );

        self.uniform_buffers.push(uniform_buffer);
        self.uniform_buffer_memories.push(uniform_buffer_memory);
    }
}}

fn dot(x: [f32;3], y: [f32;3]) -> f32 {
    return (x[0] * y[0]) + (x[1] * y[1]) + (x[2] * y[2]);
}

impl crate::engine::Engine { pub fn update_uniform_buffers(&mut self, current_image: usize) {
    let mut aspect = self.swapchain_extent.width as f32 / self.swapchain_extent.height as f32;

    let ubo = UniformBufferObject {
        model: {
            let rot = 0f32;
            let pos = [0.0, 0.0, 0.0];
            let scale = [1.0, 1.0, 1.0];

            let radians = rot.to_radians();

            let (s, c) = (radians.sin(), radians.cos());

            [[c*scale[0], s, 0.0, pos[0]],
             [-s, c*scale[1], 0.0, -pos[1]],
             [0.0, 0.0, 1.0*scale[2], pos[2]],
             [0.0, 0.0, 0.0, 1.0]]
        },
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

    let buffer_memory = self.uniform_buffer_memories[current_image];

    let size = std::mem::size_of::<UniformBufferObject>() as u64;

    let mut data_ptr: *mut std::ffi::c_void = std::ptr::null_mut();

    unsafe {vkMapMemory(
        self.device,
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

    unsafe {vkUnmapMemory(self.device, buffer_memory)};
}}

impl crate::engine::Engine { pub fn create_descriptor_pool(&mut self) {
    let pool_size = VkDescriptorPoolSize {
        type_: 6,
        descriptor_count: self.swapchain_images.len() as u32
    };

    let pool_sizes = [pool_size];

    let create_info = VkDescriptorPoolCreateInfo {
        s_type: 33,
        p_next: std::ptr::null(),
        flags: 0,
        max_sets: self.swapchain_images.len() as u32,
        pool_size_count: pool_sizes.len() as u32,
        pool_sizes: pool_sizes.as_ptr()
    };

    unsafe {vkCreateDescriptorPool(
        self.device,
        &create_info as *const VkDescriptorPoolCreateInfo, 
        std::ptr::null(),
        &mut self.descriptor_pool
    )};
}}
