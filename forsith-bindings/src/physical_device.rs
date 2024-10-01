use crate::{define_extern_functions, define_vk_bitmasks, define_vk_enums, define_vk_structs, instance::VkInstance, vk_result::VkResult, VkBool32, VkExtent3D, VkHandle};
use std::ffi::c_char;


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


define_vk_enums!(
    pub VkPhysicalDeviceType {
        VK_PHYSICAL_DEVICE_TYPE_OTHER = 0,
        VK_PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU = 1,
        VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU = 2,
        VK_PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU = 3,
        VK_PHYSICAL_DEVICE_TYPE_CPU = 4,
    }
);


define_vk_structs!(
    pub VkPhysicalDeviceProperties {
        api_version: u32,
        driver_version: u32,
        vendor_id: u32,
        device_id: u32,
        device_type: VkPhysicalDeviceType,
        device_name: [c_char; 256],
        pipeline_cache_UUID: [u8; 16],
        limits: [u8; 504],
        sparse_properties: VkPhysicalDeviceSparseProperties
    }

    pub VkPhysicalDeviceSparseProperties {
        residencyStandard2DBlockShape: VkBool32,
        residencyStandard2DMultisampleBlockShape: VkBool32,
        residencyStandard3DBlockShape: VkBool32,
        residencyAlignedMipSize: VkBool32,
        residencyNonResidentStrict: VkBool32
    }

    pub VkQueueFamilyProperties {
        queue_flags: VkQueueFlags,
        queue_count: u32,
        timestamp_valid_bits: u32,
        minImageTransferGranularity: VkExtent3D
    }
);


define_extern_functions!(["vulkan"]("C")
    pub (enumerate physical_devices: VkPhysicalDevice) vkEnumeratePhysicalDevices(
        instance: VkInstance,
    ) -> VkResult;

    pub vkGetPhysicalDeviceProperties(
        device: VkPhysicalDevice,
        physical_device_properties: *mut VkPhysicalDeviceProperties
    );

    pub (enumerate queue_family_properties: VkQueueFamilyProperties) vkGetPhysicalDeviceQueueFamilyProperties(
        device: VkPhysicalDevice,
    );
);
