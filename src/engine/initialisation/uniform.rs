use crate::vulkan::{
    uniform::{
        VkDescriptorSet,
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
        VkBuffer,
        VkDeviceMemory,
        vkMapMemory,
        vkUnmapMemory
    }
};


impl crate::engine::Engine { pub fn create_descriptor_sets(&mut self, uniform_buffers: Vec<VkBuffer>) -> Vec<VkDescriptorSet> {
    let layouts: Vec<_> = (uniform_buffers).iter().map(|_| self.descriptor_set_layout).collect();

    let allocate_info = VkDescriptorSetAllocateInfo {
        s_type: 34,
        p_next: std::ptr::null(),
        descriptor_pool: self.descriptor_pool,
        descriptor_set_count: uniform_buffers.len() as u32,
        set_layouts: layouts.as_ptr()
    };

    let mut descriptor_sets: Vec<_> = (uniform_buffers).iter().map(|_| 0).collect();


    unsafe {vkAllocateDescriptorSets(self.device, &allocate_info as *const VkDescriptorSetAllocateInfo, descriptor_sets.as_mut_ptr())};

    
    descriptor_sets.iter().zip(uniform_buffers.iter()).for_each(|(&set, &buffer)| {
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

    return descriptor_sets;
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
        max_sets: 100,
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
