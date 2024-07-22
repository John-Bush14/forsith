use crate::vulkan::{
    uniform::{
        UniformBufferObject,
        VkDescriptorSetLayoutBinding,
        VkDescriptorSetLayoutCreateInfo,
        vkCreateDescriptorSetLayout
    },
    vertex::{
        vkMapMemory,
        vkUnmapMemory
    }
};


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

impl crate::engine::Engine { pub fn update_uniform_buffers(&mut self) {
    let aspect = self.swapchain_extent.width as f32 / self.swapchain_extent.height as f32;

    let ubo = UniformBufferObject {
        model: {
            let radians = std::f32::consts::PI / 2.0;

            let (s, c) = (radians.sin(), radians.cos());

            [[c, s, 0.0, 0.0],
            [-s, c, 0.0, 0.0],
             [0.0, 0.0, 0.0, 0.0],
             [0.0, 0.0, 0.0, 0.0]]
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
             [-dot(xaxis, eye), -dot(yaxis, eye), -dot(zaxis, eye), 1.0]]
        },
        proj: {
            let f = 1f32/(5f32.to_radians()/2f32).tan();
            
            let near = 0.1;
            let far = 10.0;

            [[f/aspect, 0.0, 0.0, 0.0],
             [0.0, -f, 0.0, 0.0],
             [0.0, 0.0, -far/(far-near), -1.0],
             [0.0, 0.0, -(far * near) / (far - near), 0.0]]
        } 
    };

    let buffer_memory = self.uniform_buffer_memories[self.current_frame];

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
