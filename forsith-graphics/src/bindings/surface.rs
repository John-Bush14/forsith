use crate::{define_extern_functions, define_vk_bitmasks, define_vk_structs, instance::VkInstance, physical_device::VkPhysicalDevice, structure_type::VkStructureType, vk_result::VkResult, VkAllocationCallbacks, VkBool32, VkHandle};


pub type VkSurfaceKHR = VkHandle;


define_vk_bitmasks!(
    pub VkHeadlessSurfaceCreateFlagsEXT(VkHeadlessSurfaceCreateFlagBitsEXT) {
        FUTURE_USE = 0
    }
);


define_vk_structs!(
    pub VkHeadlessSurfaceCreateInfoEXT(VkStructureType::VkStructureTypeHeadlessSurfaceCreateInfoExt) {
        flags: VkHeadlessSurfaceCreateFlagsEXT
    }
);


define_extern_functions!(["vulkan"]("C")
    pub vkCreateHeadlessSurfaceEXT(
        instance: VkInstance,
        create_info: *const VkHeadlessSurfaceCreateInfoEXT,
        allocator: *const VkAllocationCallbacks,
        surface: *mut VkSurfaceKHR
    ) -> VkResult;

    pub vkGetPhysicalDeviceSurfaceSupportKHR(
        physical_device: VkPhysicalDevice,
        queue_family_index: u32,
        surface: VkSurfaceKHR,
        supported: *mut VkBool32
    ) -> VkResult;
);
