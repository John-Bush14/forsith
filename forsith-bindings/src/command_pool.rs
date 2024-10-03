use crate::{define_extern_functions, define_vk_bitmasks, define_vk_structs, device::VkDevice, physical_device::VkQueueFamily, structure_type::VkStructureType, vk_result::VkResult, VkAllocationCallbacks, VkHandle};


pub type VkCommandPool = VkHandle;

pub type VkCommandBuffer = VkHandle;


define_vk_bitmasks!(
    pub VkCommandPoolCreateFlags(VkCommandPoolCreateFlagBits) {
        VK_COMMAND_POOL_CREATE_TRANSIENT_BIT = 0x00000001,
        VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT = 0x00000002,

        // Provided by VK_VERSION_1_1
        VK_COMMAND_POOL_CREATE_PROTECTED_BIT = 0x00000004,
    }
);


define_vk_structs!(
    pub VkCommandPoolCreateInfo(VkStructureType::VkStructureTypeCommandPoolCreateInfo) {
        flags: VkCommandPoolCreateFlags,
        queue_family: VkQueueFamily
    }
);


define_extern_functions!(["vulkan"]("C")
    pub vkCreateCommandPool(
        device: VkDevice,
        create_info: *const VkCommandPoolCreateInfo,
        allocator: *const VkAllocationCallbacks,
        command_pool: *mut VkCommandPool
    ) -> VkResult;

    pub vkDestroyCommandPool(
        device: VkDevice,
        command_pool: VkCommandPool,
        allocator: *const VkAllocationCallbacks
    );
);
