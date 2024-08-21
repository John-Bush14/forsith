use crate::vulkan::{image::{
        vkBindImageMemory, vkCmdCopyBufferToImage, vkCmdPipelineBarrier, vkCreateImage, vkCreateImageView, vkCreateSampler, vkGetImageMemoryRequirements, VkBufferImageCopy, VkComponentMapping, VkExtent3D, VkImage, VkImageCreateInfo, VkImageMemoryBarrier, VkImageSubresourceLayers, VkImageSubresourceRange, VkImageView, VkImageViewCreateInfo, VkOffset3D, VkSampler, VkSamplerCreateInfo
    }, vertex::{vkAllocateMemory, vkDestroyBuffer, vkFreeMemory, vkGetPhysicalDeviceMemoryProperties, vkMapMemory, vkUnmapMemory, VkBuffer, VkDeviceMemory, VkMemoryAllocateInfo, VkMemoryRequirements, VkPhysicalDeviceMemoryProperties}};


impl crate::engine::Engine {pub fn create_image_usr(&mut self, file: String) -> (VkImage, VkImageView, VkSampler) {
    let (image, _mem) = self.create_texture_image(file);
    
    let image_view = self.create_image_view(image, 0x00000001, 37);
    println!("image_view: {}", image_view);

    let sampler = self.create_texture_sampler();

    return (image, image_view, sampler);
}}

impl super::Engine {pub fn create_texture_image(&self, file: String) -> (VkImage, VkDeviceMemory) {
    let image = image::open(file).expect("error getting image");
    let image_as_rgb = image.to_rgba();
    let width = (&image_as_rgb).width();
    let height = (&image_as_rgb).height();
    let pixels: Vec<u8> = image_as_rgb.into_raw();
    let image_size = (pixels.len() * std::mem::size_of::<u8>()) as u64;


    let (buffer, memory, _) = self.create_buffer(image_size, 0x00000001, 0x00000002 | 0x00000004);

    let mut data_ptr: *mut std::ffi::c_void = std::ptr::null_mut();

    unsafe {vkMapMemory(
        self.device,
        memory,
        0,
        image_size,
        0,
        &mut data_ptr as _
    )};

    unsafe {std::ptr::copy_nonoverlapping(pixels.as_ptr(), data_ptr as _, pixels.len())};

    unsafe {vkUnmapMemory(self.device, memory)};


    let (image, image_memory) = self.create_image(width, height, 37, 0, 0x00000002 | 0x00000004, 0x00000001);


    self.transition_image_layout(image, 37, 0, 7);


    self.copy_buffer_to_image(buffer, image, width, height);
    

    self.transition_image_layout(image, 37, 7, 5);


    unsafe {
        vkDestroyBuffer(self.device, buffer, std::ptr::null());
        vkFreeMemory(self.device, memory, std::ptr::null())
    }

    return (image, image_memory);
}}

impl super::Engine {pub fn create_image(&self, 
    width: u32,
    height: u32, 
    format: u32,
    tiling: u32,
    usage: u32,
    mem_properties: u32,
) -> (VkImage, VkDeviceMemory) {
    let mut device_memory_properties: VkPhysicalDeviceMemoryProperties = unsafe {std::mem::zeroed()};

    unsafe {
        vkGetPhysicalDeviceMemoryProperties(self.physical_device, &mut device_memory_properties as &mut VkPhysicalDeviceMemoryProperties);
    }


    let create_info = VkImageCreateInfo {
        s_type: 14,
        p_next: std::ptr::null(),
        flags: 0,
        image_type: 1,
        format,
        extent: VkExtent3D {
            width,
            height,
            depth: 1
        },
        miplevels: 1,
        array_layers: 1,
        samples: 0x00000001,
        tiling,
        usage,
        sharing_mode: 0,
        queue_family_index_count: 0,
        queue_family_indices: std::ptr::null(),
        initial_layout: 0,
    };


    let mut image = 0;

    unsafe {vkCreateImage(self.device, &create_info as *const VkImageCreateInfo, std::ptr::null(), &mut image)};


    let mut memory_requirements: VkMemoryRequirements = unsafe {std::mem::zeroed()};

    unsafe {vkGetImageMemoryRequirements(self.device, image, &mut memory_requirements as *mut VkMemoryRequirements)};

    let mem_type_index = self.find_memory_type(device_memory_properties, &memory_requirements, mem_properties);


    let alloc_info = VkMemoryAllocateInfo {
        s_type: 5,
        p_next: std::ptr::null(),
        allocation_size: memory_requirements.size,
        memory_type_index: mem_type_index,
    };

    let mut memory: VkDeviceMemory = 0;

    unsafe {vkAllocateMemory(self.device, &alloc_info as *const VkMemoryAllocateInfo, std::ptr::null(), &mut memory)};

    unsafe {vkBindImageMemory(self.device, image, memory, 0)};
        

    return (image, memory);
}}

impl crate::engine::Engine { pub fn create_image_view(&mut self, image: VkImage, aspect_mask: u32, format: u32) -> VkImageView { unsafe {
    let components = VkComponentMapping {r: 0, g: 0, b: 0, a: 0};

    let subresource_range = VkImageSubresourceRange {
        aspect_mask,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1
    };
        
    let image_view_create_info = VkImageViewCreateInfo {
        s_type: 15,
        p_next: std::ptr::null(),
        flags: 0,
        image,
        view_type: 1, // dimensions -1 (1 = 2D)
        format,
        components,
        subresource_range
    };


    let mut image_view: VkImageView = 0;

    vkCreateImageView(self.device, &image_view_create_info as *const VkImageViewCreateInfo, std::ptr::null(), &mut image_view);


    return image_view;
}}}

impl crate::engine::Engine {pub fn create_texture_sampler(&mut self) -> VkSampler {
    let create_info = VkSamplerCreateInfo {
        s_type: 31,
        p_next: std::ptr::null(),
        flags: 0,
        mag_filter: 1,
        min_filter: 1,
        mipmap_mode: 1,
        address_mode_u: 0,
        addres_mode_v: 0,
        address_mode_w: 0,
        mip_lod_bias: 0.0,
        anisotropy_enable: 1,
        max_anisotropy: 16.0,
        compare_enable: 0,
        compare_op: 7,
        min_lod: 0.0,
        max_lod: 0.0,
        border_color: 3,
        unnormalized_coordinates: 0,
    };

    let mut sampler = 0;

    unsafe {vkCreateSampler(self.device, &create_info as *const VkSamplerCreateInfo, std::ptr::null(), &mut sampler)};

    return sampler;
}}

impl crate::engine::Engine {pub fn create_swapchain_image_views(&mut self) {
    self.swapchain_image_views = self.swapchain_images.clone().iter().map(|image| self.create_image_view(*image, 0x00000001, self.swapchain_image_format.format)).collect();              
}}

impl crate::engine::Engine {pub fn transition_image_layout(&self, image: VkImage, format: u32, old_layout: u32, new_layout: u32) {
    self.execute_one_time_command(self.command_pool, self.graphics_queue, |cmd_buffer| {
        let (src_access_mask, dst_access_mask, src_stage, dst_stage) = match (old_layout, new_layout) {
            (0, 3) => (
                0,
                0x00000200 | 0x00000400,
                0x00000001,
                0x00000100,
            ),

            (0, 7) => (
                0,
                0x00001000,
                0x00000001,
                0x00001000
            ),

            (7, 5) => (
                0x00001000,
                0x00000020,
                0x00001000,
                0x00000080
            ),

            _ => (0, 0, 0, 0),
        };


        let aspect_mask = if new_layout == 3 {
            let mut mask = 0x00000002;

            if format == 130 || format == 129 {
                mask = mask | 0x00000004;
            }

            mask
        } else {0x00000001};


        let barrier = VkImageMemoryBarrier {
            s_type: 45,
            p_next: std::ptr::null(),
            src_access_mask,
            dst_access_mask,
            old_layout,
            new_layout,
            src_queue_family_index: std::u32::MAX,
            dst_queue_family_index: std::u32::MAX,
            image,
            subresource_range: VkImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        };

        let barriers = [barrier];

        unsafe {vkCmdPipelineBarrier(
            cmd_buffer,
            src_stage,
            dst_stage,
            0,
            0, 
            std::ptr::null(),
            0,
            std::ptr::null(),
            barriers.len() as u32,
            barriers.as_ptr()
        )};
    });
}}

impl crate::engine::Engine { pub fn create_depth_image(&mut self) {
    (self.depth_image.0, self.depth_image.1) = self.create_image(
        self.swapchain_extent.width,
        self.swapchain_extent.height,
        self.depth_format,
        0,
        0x00000020,
        0x00000001,
    );


    self.transition_image_layout(self.depth_image.0, self.depth_format, 0, 3);

    self.depth_image.2 = self.create_image_view(self.depth_image.0, 0x00000002, self.depth_format)
}}

impl crate::engine::Engine {pub fn copy_buffer_to_image(&self, buffer: VkBuffer, image: VkImage, width: u32, height: u32) {
    self.execute_one_time_command(self.command_pool, self.graphics_queue, |cmd_buffer| {
        let region = VkBufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: VkImageSubresourceLayers {
                aspect_mask: 0x00000001,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_offset: VkOffset3D {x: 0, y: 0, z: 0},
            image_extent: VkExtent3D {width, height, depth: 1},
        };

        unsafe {vkCmdCopyBufferToImage(cmd_buffer, buffer, image, 7, 1, &region as *const VkBufferImageCopy)};
    })
}}
