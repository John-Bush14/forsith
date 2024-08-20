use crate::vulkan::{image::{
        vkBindImageMemory, vkCmdPipelineBarrier, vkCreateImage, vkCreateImageView, vkGetImageMemoryRequirements, VkComponentMapping, VkExtent3D, VkImage, VkImageCreateInfo, VkImageMemoryBarrier, VkImageSubresourceRange, VkImageView, VkImageViewCreateInfo
    }, vertex::{vkAllocateMemory, vkGetPhysicalDeviceMemoryProperties, VkDeviceMemory, VkMemoryAllocateInfo, VkMemoryRequirements, VkPhysicalDeviceMemoryProperties}};

impl super::Engine {pub fn create_texture_image(&self, file: String) {
    let image = image::open(file).expect("error getting image");
    let image_as_rgb = image.to_rgba();
    let _image_width = (&image_as_rgb).width();
    let _image_height = (&image_as_rgb).height();
    let pixels = image_as_rgb.into_raw();
    let _image_size = (pixels.len() * std::mem::size_of::<u8>()) as u64;
}}

impl super::Engine {pub fn create_image(&mut self, 
    width: u32,
    height: u32, 
    format: u32,
    tiling: u32,
    usage: u32,
    mem_properties: u32,
    device_mem_properties: VkPhysicalDeviceMemoryProperties
) -> (VkImage, VkDeviceMemory) {
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

    let mem_type_index = self.find_memory_type(device_mem_properties, &memory_requirements, mem_properties);


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

impl crate::engine::Engine { pub fn create_image_view(&mut self, image: VkImage, aspect_mask: u32) -> VkImageView { unsafe {
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
        format: self.swapchain_image_format.format,
        components,
        subresource_range
    };


    let mut image_view: VkImageView = 0;

    vkCreateImageView(self.device, &image_view_create_info as *const VkImageViewCreateInfo, std::ptr::null(), &mut image_view);


    return image_view;
}}}

impl crate::engine::Engine {pub fn create_swapchain_image_views(&mut self) {
    self.swapchain_image_views = self.swapchain_images.clone().iter().map(|image| self.create_image_view(*image, 0x00000001)).collect();              
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
            _ => (0, 0, 0, 0)
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

impl crate::engine::Engine { pub fn create_depth_resources(&mut self) {
    let mut device_memory_properties: VkPhysicalDeviceMemoryProperties = unsafe {std::mem::zeroed()};

    unsafe {
        vkGetPhysicalDeviceMemoryProperties(self.physical_device, &mut device_memory_properties as &mut VkPhysicalDeviceMemoryProperties);
    }


    (self.depth_resource.0, self.depth_resource.1) = self.create_image(
        self.swapchain_extent.width,
        self.swapchain_extent.height,
        self.depth_format,
        0,
        0x00000020,
        0x00000001,
        device_memory_properties
    );


    self.transition_image_layout(self.depth_resource.0, self.depth_format, 0, 3);

    self.depth_resource.2 = self.create_image_view(self.depth_resource.0, 0x00000002)
}}
