use crate::vulkan::swapchain::image_view::{
            VkImageView,
            VkComponentMapping,
            VkImageViewCreateInfo,
            VkImageSubresourceRange,
            vkCreateImageView
};


impl crate::engine::Engine { pub fn create_image_views(&mut self) { unsafe {
    self.swapchain_image_views = self.swapchain_images.iter().map(|image|{
        let components = VkComponentMapping {r: 0, g: 0, b: 0, a: 0};

        let subresource_range = VkImageSubresourceRange {
            aspect_mask: 0x00000001,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1
        };
        
        let image_view_create_info = VkImageViewCreateInfo {
            s_type: 15,
            p_next: std::ptr::null(),
            flags: 0,
            image: *image,
            view_type: 1, // dimensions -1 (1 = 2D)
            format: self.swapchain_image_format.format,
            components,
            subresource_range
        };


        let mut image_view: VkImageView = 0;

        vkCreateImageView(self.device, &image_view_create_info as *const VkImageViewCreateInfo, std::ptr::null(), &mut image_view);

        return image_view
    }).collect();
}}}
