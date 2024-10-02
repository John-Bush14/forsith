use std::ffi::{c_char, c_void};
use crate::{define_extern_functions, define_vk_bitmasks, define_vk_enums, define_vk_structs, physical_device::VkPhysicalDevice, structure_type::VkStructureType, vk_result::VkResult, VkAllocationCallbacks, VkHandle};


pub type VkDevice = VkHandle;


define_vk_bitmasks!(
    pub VkDeviceCreateFlags(VkDeviceCreateFlagBits) {
        FUTURE_USE = 0
    }

    pub VkDeviceQueueCreateFlags(VKDeviceQueueCreateFlagBits) {
        NO_CLUE = 0
    }
);


define_vk_structs!(
    pub VkDeviceQueueCreateInfo(VkStructureType::VkStructureTypeDeviceQueueCreateInfo) {
        flags: VkDeviceQueueCreateFlags,
        queue_family_index: u32,
        queue_count: u32,
        queue_priorities: *const f32
    }

    pub VkDeviceCreateInfo(VkStructureType::VkStructureTypeDeviceCreateInfo) {
        flags: VkDeviceCreateFlags,
        queue_create_info_count: u32,
        queue_create_infos: *const VkDeviceQueueCreateInfo,
        enabled_layer_count: u32,
        enabled_layer_names: *const *const c_char,
        enabled_extension_count: u32,
        enabled_extensions: *const *const c_char,
        enabled_features: *const c_void
    }
);


define_extern_functions!(["vulkan"]("C")
    pub vkCreateDevice(
        physical_device: VkPhysicalDevice,
        create_info: *const VkDeviceCreateInfo,
        allocator: *const VkAllocationCallbacks,
        device: *mut VkDevice
    ) -> VkResult;
);
