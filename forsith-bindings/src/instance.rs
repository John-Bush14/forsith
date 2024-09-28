use crate::{define_extern_functions, define_vk_bitmasks, define_vk_structs, structure_type::VkStructureType, vk_result::VkResult, VkAllocationCallbacks, VkHandle, VkVersion};
use std::ffi::c_char;


pub type VkInstance = VkHandle;


define_vk_bitmasks!(
    pub VkInstanceCreateFlags(VkInstanceCreateFlagBits) {
        VK_INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT_KHR = 0x00000001
    }
);


define_vk_structs!(
    pub VkApplicationInfo(VkStructureType::VkStructureTypeApplicationInfo) {
        application_name: *const c_char,
        application_version: VkVersion,
        engine_name: *const c_char,
        engine_version: VkVersion,
        api_version: VkVersion
    }

    pub VkInstanceCreateInfo(VkStructureType::VkStructureTypeInstanceCreateInfo) {
        flags: VkInstanceCreateFlags,
        application_info: *const VkApplicationInfo,
        enabled_layer_count: u32,
        enabled_layer_names: *const *const c_char,
        enabled_extension_count: u32,
        enabled_extensions: *const *const c_char
    }
);


define_extern_functions!(["vulkan"]("C")
    pub vkCreateInstance(
        create_info: *const VkInstanceCreateInfo,
        allocator: *const VkAllocationCallbacks,
        instance: *mut VkInstance
    ) -> VkResult;

    pub vkDestroyInstance(
        instance: VkInstance,
        allocator: *const VkAllocationCallbacks
    );
);
