use crate::vulkan::{
    image::{Texture, VkDescriptorImageInfo, VkImage, VkImageView, VkSampler}, pipeline::ShaderItem, uniform::{
        vkAllocateDescriptorSets, vkCreateDescriptorPool, vkCreateDescriptorSetLayout, vkUpdateDescriptorSets, DescriptorBindings, VkDescriptorBufferInfo, VkDescriptorPoolCreateInfo, VkDescriptorPoolSize, VkDescriptorSet, VkDescriptorSetAllocateInfo, VkDescriptorSetLayout, VkDescriptorSetLayoutBinding, VkDescriptorSetLayoutCreateInfo, VkWriteDescriptorSet
    }, vertex::{
        VkBuffer,
        VkDeviceMemory
    }
};


impl crate::engine::Engine { pub(crate) fn create_uniform_buffers(&self, size: u64) -> (Vec<VkBuffer>, Vec<VkDeviceMemory>) {
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

impl crate::engine::Engine { pub(crate) fn create_descriptor_sets(
    &mut self,
    uniform_buffers_len: usize,
    descriptor_set_layout: VkDescriptorSetLayout
) -> Vec<VkDescriptorSet> {

    let layouts: Vec<_> = (0 .. uniform_buffers_len).collect::<Vec<_>>().iter().map(|_| descriptor_set_layout).collect();

    let allocate_info = VkDescriptorSetAllocateInfo {
        s_type: 34,
        p_next: std::ptr::null(),
        descriptor_pool: self.descriptor_pool,
        descriptor_set_count: uniform_buffers_len as u32,
        set_layouts: layouts.as_ptr()
    };

    let mut descriptor_sets: Vec<_> = (0 .. uniform_buffers_len).collect::<Vec<_>>().iter().map(|_| 0).collect();

    unsafe {vkAllocateDescriptorSets(self.device, &allocate_info as *const VkDescriptorSetAllocateInfo, descriptor_sets.as_mut_ptr())};


    return descriptor_sets;
}}


impl crate::engine::Engine { pub(crate) fn update_descriptor_sets(
    &mut self,
    descriptor_sets: &Vec<VkDescriptorSet>,
    bindings: Vec<(u32, Vec<VkBuffer>, u64)>, // binding, buffer range (object size)
    uniforms: Vec<ShaderItem>
) {
    for i in 0 .. descriptor_sets.len() {
        let mut descriptor_writes = vec!();
        let mut buffer_infos = vec!();
        let mut image_infos = vec!();

        println!("bindings: {:?}", bindings);

        bindings.iter().for_each(|(binding, buffers, buffer_range)| {
            let buffer_info = VkDescriptorBufferInfo {
                buffer: if buffers.len() > 0 {buffers[i]} else {0},
                offset: 0,
                range: *buffer_range
            }; buffer_infos.push(buffer_info);

            let mut descriptor_write = VkWriteDescriptorSet {
                s_type: 35,
                p_next: std::ptr::null(),
                dst_set: descriptor_sets[i],
                dst_binding: *binding,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: 6,
                image_info: std::ptr::null(),
                buffer_info: &buffer_infos[buffer_infos.len()-1] as *const VkDescriptorBufferInfo,
                texel_buffer_view: std::ptr::null()
            };

            match &uniforms[*binding as usize] {
                ShaderItem::Sampler2D(texture) => {
                    let image_info = VkDescriptorImageInfo {
                        sampler: texture.sampler,
                        image_view: texture.image_view,
                        image_layout: 5
                    }; image_infos.push(image_info);

                    descriptor_write.image_info = &image_infos[image_infos.len()-1] as *const VkDescriptorImageInfo;
                    descriptor_write.descriptor_type = 1;
                    descriptor_write.buffer_info = std::ptr::null()
                },
                ShaderItem::Void => {}
            }

            descriptor_writes.push(descriptor_write);
        });

        unsafe {vkUpdateDescriptorSets(
            self.device,
            descriptor_writes.len() as u32,
            descriptor_writes.as_ptr(),
            0,
            std::ptr::null()
        )};
    };
}}

impl crate::engine::Engine { pub(crate) fn create_descriptor_set_layout(&self, descriptor_bindings: &DescriptorBindings) -> VkDescriptorSetLayout {
    let vertex_binding = VkDescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: 6,
        descriptor_count: 1,
        stage_flags: 0x00000001,
        immutable_samplers: std::ptr::null()
    };

    let fragment_binding = VkDescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: 1,
        descriptor_count: 1,
        stage_flags: 0x00000010,
        immutable_samplers: std::ptr::null()
    };

    let mut bindings = vec!();

    for i in 0 .. descriptor_bindings.vertex as usize {
        let mut binding = vertex_binding.clone();
        binding.binding = i as u32;
        bindings.push(binding);
    }

    for i in 0 .. descriptor_bindings.fragment as usize {
        let mut binding = fragment_binding.clone();
        binding.binding = i as u32 + descriptor_bindings.vertex;
        bindings.push(binding);
    }

    let create_info = VkDescriptorSetLayoutCreateInfo {
        s_type: 32,
        p_next: std::ptr::null(),
        flags: 0,
        binding_count: bindings.len() as u32,
        bindings: bindings.as_ptr()
    };

    let mut descriptor_set_layout = 0;

    unsafe {vkCreateDescriptorSetLayout(
        self.device,
        &create_info as *const VkDescriptorSetLayoutCreateInfo,
        std::ptr::null(),
        &mut descriptor_set_layout
    )};

    return descriptor_set_layout;
}}

impl crate::engine::Engine { pub(crate) fn create_descriptor_pool(&mut self) {
    let ubo_pool_size = VkDescriptorPoolSize {
        type_: 6,
        descriptor_count: self.swapchain_images.len() as u32
    };

    let sampler_pool_size = VkDescriptorPoolSize {
        type_: 1,
        descriptor_count: self.swapchain_images.len() as u32
    };

    let pool_sizes = [ubo_pool_size, sampler_pool_size];

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
