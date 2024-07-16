use crate::vulkan::{
    swapchain::{
        VkImage,
        VkExtent2D,
        VkSwapchainKHR,
        VkSwapchainCreateInfo,
        VkSurfaceCapabilitiesKHR,
        vkCreateSwapchainKHR,
        vkGetSwapchainImagesKHR,
        vkGetPhysicalDeviceSurfacePresentModesKHR,
        vkGetPhysicalDeviceSurfaceCapabilitiesKHR
    }
};

use crate::{
    vk_enumerate_to_vec
};


impl crate::engine::Engine { pub fn create_swapchain(&mut self, presentation_queue: u32, graphics_queue: u32) { unsafe {
    let present_modes = vk_enumerate_to_vec!(vkGetPhysicalDeviceSurfacePresentModesKHR, u32, self.physical_device, self.surface_khr,);

    let present_mode = {if present_modes.contains(&1) {1} else {2}};
    
    
    let mut capabilities: VkSurfaceCapabilitiesKHR = std::mem::zeroed();

    vkGetPhysicalDeviceSurfaceCapabilitiesKHR(self.physical_device, self.surface_khr, &mut capabilities as *mut VkSurfaceCapabilitiesKHR);

    let extent = {
        if capabilities.current_extent.width != std::u32::MAX {capabilities.current_extent}
        else {
            let min = capabilities.min_image_extent;
            let max = capabilities.max_image_extent;
            let width = 800.min(max.width).max(min.width);
            let height = 600.min(max.height).max(min.height);
            VkExtent2D { width, height }
        }
    };

    let image_count = {
        let max = capabilities.max_image_count;
        let mut preferred = capabilities.min_image_count + 1;
        if max > 0 {preferred = preferred.max(max);}
        preferred
    };

    let transform = capabilities.current_transform;

    let format = 37;


    let queue_family_indices = {
        if presentation_queue == graphics_queue {vec![graphics_queue]} 
        else {vec![presentation_queue, graphics_queue]}
    };

    let swapchain_create_info: VkSwapchainCreateInfo = VkSwapchainCreateInfo {
        s_type: 1000001000,
        p_next: std::ptr::null(),
        flags: 0,
        surface: self.surface_khr,
        min_image_count: image_count,
        image_format: format,
        image_color_space: 0,
        image_extent: extent.clone(),
        image_array_layers: 1,
        image_usage: 0x00000010,
        image_sharing_mode: (queue_family_indices.len()-1) as u32,
        queue_family_index_count: queue_family_indices.len() as u32,
        queue_family_indices: queue_family_indices.as_ptr(),
        pre_transform: transform,
        composite_alpha: 0x00000001,
        present_mode: present_mode,
        clipped: 1,
        old_swapchain: 0
    };


    let mut swapchain: VkSwapchainKHR = std::mem::zeroed();

    vkCreateSwapchainKHR(
        self.device, 
        &swapchain_create_info as *const VkSwapchainCreateInfo,
        std::ptr::null(),
        &mut swapchain as *mut VkSwapchainKHR
    );


    let images = vk_enumerate_to_vec!(vkGetSwapchainImagesKHR, VkImage, self.device, swapchain,);


    self.swapchain = swapchain;
    self.swapchain_image_format = format;
    self.swapchain_images = images;
    self.swapchain_extent = extent;
}}}
