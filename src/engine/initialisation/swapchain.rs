use crate::vulkan::{
    swapchain::{
        VkExtent2D,
        VkSwapchainKHR,
        VkSurfaceFormatKHR,
        VkSwapchainCreateInfo,
        VkSurfaceCapabilitiesKHR,
        vkCreateSwapchainKHR,
        vkGetSwapchainImagesKHR,
        vkGetPhysicalDeviceSurfaceFormatsKHR,
        vkGetPhysicalDeviceSurfacePresentModesKHR,
        vkGetPhysicalDeviceSurfaceCapabilitiesKHR
    },
    image::VkImage
};

use crate::vk_enumerate_to_vec;


impl crate::engine::Engine { pub fn create_swapchain(&mut self) { unsafe {
    let present_modes = vk_enumerate_to_vec!(vkGetPhysicalDeviceSurfacePresentModesKHR, u32, self.physical_device, self.surface_khr,);

    let present_mode = {if present_modes.contains(&1) {1} else {2}};
    
    
    let mut capabilities: VkSurfaceCapabilitiesKHR = std::mem::zeroed();

    vkGetPhysicalDeviceSurfaceCapabilitiesKHR(self.physical_device, self.surface_khr, &mut capabilities as *mut VkSurfaceCapabilitiesKHR);
    
    let extent = {
        if capabilities.current_extent.width == std::u32::MAX {VkExtent2D { width: self.dimensions[0] as u32, height: self.dimensions[1] as u32}}
        else {
            let min = capabilities.min_image_extent;
            let max = capabilities.max_image_extent;
            let width = (self.dimensions[0] as u32).min(max.width).max(min.width);
            let height = (self.dimensions[1] as u32).min(max.height).max(min.height);
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

    let surface_format = {
        let supported_formats = vk_enumerate_to_vec!(
            vkGetPhysicalDeviceSurfaceFormatsKHR, 
            VkSurfaceFormatKHR, 
            self.physical_device,
            self.surface_khr,
        );

        if !(supported_formats.len() <= 1 && supported_formats[0].format == 0) {
            supported_formats.iter().find(|format| format.format == 37 && format.color_space == 0).unwrap_or(&supported_formats[0]).clone()
        } else {
            VkSurfaceFormatKHR {format: 37, color_space: 0}
        }
    };

    println!("{:?}", surface_format);

    let queue_family_indices = {
        if self.presentation_family == self.graphics_family {vec![self.graphics_family]} 
        else {vec![self.presentation_family, self.graphics_family]}
    };

    let swapchain_create_info: VkSwapchainCreateInfo = VkSwapchainCreateInfo {
        s_type: 1000001000,
        p_next: std::ptr::null(),
        flags: 0,
        surface: self.surface_khr,
        min_image_count: image_count,
        image_format: surface_format.format,
        image_color_space: surface_format.color_space,
        image_extent: extent.clone(),
        image_array_layers: 1,
        image_usage: 0x00000010,
        image_sharing_mode: (queue_family_indices.len()-1) as u32,
        queue_family_index_count: queue_family_indices.len() as u32,
        queue_family_indices: queue_family_indices.as_ptr(),
        pre_transform: transform,
        composite_alpha: 0x00000001,
        present_mode,
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
    self.swapchain_image_format = surface_format;
    self.swapchain_images = images;
    self.swapchain_extent = extent;
}}}
