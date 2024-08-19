use crate::vulkan::{image::{
        vkCreateImageView, VkComponentMapping, VkImage, VkImageSubresourceRange, VkImageView, VkImageViewCreateInfo
    }, vertex::VkPhysicalDeviceMemoryProperties};

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
) -> VkImage {todo!();  
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

    return image_view
}}}

impl crate::engine::Engine {pub fn create_swapchain_image_views(&mut self) {
    self.swapchain_image_views = self.swapchain_images.clone().iter().map(|image| self.create_image_view(*image, 0x00000001)).collect();              
}}
