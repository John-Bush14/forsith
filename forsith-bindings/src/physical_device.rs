use crate::{define_vk_bitmasks, VkHandle};


pub type VkPhysicalDevice = VkHandle;

pub type VkQueueFamily = VkHandle;

pub type VkQueue = VkHandle;


define_vk_bitmasks!(
    pub VkQueueFlags(VkQueueFlagBits) {
        VK_QUEUE_GRAPHICS_BIT = 0x00000001,
        VK_QUEUE_COMPUTE_BIT = 0x00000002,
        VK_QUEUE_TRANSFER_BIT = 0x00000004,
        VK_QUEUE_SPARSE_BINDING_BIT = 0x00000008,
        // Provided by VK_VERSION_1_1
        VK_QUEUE_PROTECTED_BIT = 0x00000010,
        // Provided by VK_KHR_video_decode_queue
        VK_QUEUE_VIDEO_DECODE_BIT_KHR = 0x00000020,
        // Provided by VK_KHR_video_encode_queue
        VK_QUEUE_VIDEO_ENCODE_BIT_KHR = 0x00000040,
        // Provided by VK_NV_optical_flow
        VK_QUEUE_OPTICAL_FLOW_BIT_NV = 0x00000100,
    }
);
