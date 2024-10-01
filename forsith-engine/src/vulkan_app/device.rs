use bindings::{device::VkDevice, physical_device::{self, vk_enumerate_physical_devices, vk_get_physical_device_properties, VkPhysicalDevice, VkPhysicalDeviceProperties, VkPhysicalDeviceType, VkQueue, VkQueueFamily, VkQueueFlagBits}};

use super::VulkanApp;


pub struct Queue {
    family: VkQueueFamily,
    queue: VkQueue,
    flags: VkQueueFlagBits
}

pub struct Device {
    physical_device: VkPhysicalDevice,
    queues: Vec<Queue>,
    device: VkDevice
}


fn rate_device_type(device_type: VkPhysicalDeviceType) -> u32 {
    match device_type {
        VkPhysicalDeviceType::VkPhysicalDeviceTypeOther => 2,
        VkPhysicalDeviceType::VkPhysicalDeviceTypeIntegratedGpu => 10,
        VkPhysicalDeviceType::VkPhysicalDeviceTypeDiscreteGpu => 6,
        VkPhysicalDeviceType::VkPhysicalDeviceTypeVirtualGpu => 8,
        VkPhysicalDeviceType::VkPhysicalDeviceTypeCpu => 4,
    }
}


impl VulkanApp {
    pub(crate) fn create_device(&self, queue_family_qualifiers: Vec<fn(VkPhysicalDevice, VkQueueFamily) -> bool>) -> Device {
        let physical_devices: Vec<VkPhysicalDevice> = vk_enumerate_physical_devices(self.instance);


        todo!();
    }
}
