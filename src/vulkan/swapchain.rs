use crate::vulkan::{
    devices::{
        physical_device::{
            VkPhysicalDevice
        },
        device::{
            VkDevice
        }
    },
    window::{
        VkSurfaceKHR
    },
    VkResult,
    VkBool32,
    VkStructureType
};

use std::ffi::{
    c_void
};


#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct VkExtent2D {
    pub width: u32,
    pub height: u32
}

#[repr(C)]
pub struct VkSurfaceCapabilitiesKHR {
    pub min_image_count: u32,
    pub max_image_count: u32,
    pub current_extent: VkExtent2D,
    pub min_image_extent: VkExtent2D,
    pub max_image_extent: VkExtent2D,
    max_image_array_layers: u32,
    supported_transforms: u32,
    pub current_transform: u32,
    supported_composite_alpha: u32,
    supported_usage_flags: u32
}

pub type VkImage = u64;

#[repr(C)]
pub struct VkSwapchainCreateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub surface: VkSurfaceKHR,
    pub min_image_count: u32,
    pub image_format: u32,
    pub image_color_space: u32,
    pub image_extent: VkExtent2D,
    pub image_array_layers: u32,
    pub image_usage: u32,
    pub image_sharing_mode: u32,
    pub queue_family_index_count: u32,
    pub queue_family_indices: *const u32,
    pub pre_transform: u32,
    pub composite_alpha: u32,
    pub present_mode: u32,
    pub clipped: VkBool32,
    pub old_swapchain: VkSwapchainKHR
}

pub type VkSwapchainKHR = u64;


#[link(name = "vulkan")]
extern "C" {
    pub fn vkCreateSwapchainKHR(
        device: VkDevice,
        creation_info: *const VkSwapchainCreateInfo,
        _: *const c_void,
        swapchain: *mut VkSwapchainKHR
    ) -> VkResult;

    pub fn vkGetPhysicalDeviceSurfacePresentModesKHR(
        physical_device: VkPhysicalDevice,
        surface: VkSurfaceKHR,
        present_mod_count: *mut u32,
        present_modes: *mut u32
    ) -> VkResult; 

    pub fn vkGetPhysicalDeviceSurfaceCapabilitiesKHR(
        physical_device: VkPhysicalDevice,
        surface: VkSurfaceKHR,
        capabilities: *mut VkSurfaceCapabilitiesKHR
    ) -> VkResult;

    pub fn vkGetSwapchainImagesKHR(
        device: VkDevice,
        swapchain: VkSwapchainKHR,
        image_count: *mut u32,
        images: *mut VkImage
    ) -> VkResult;
}
