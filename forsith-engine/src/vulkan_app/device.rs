use bindings::{device::VkDevice, physical_device::{VkPhysicalDevice, VkQueue, VkQueueFamily, VkQueueFlagBits}};

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


impl VulkanApp {
    pub(crate) fn create_device(queue_family_qualifiers: Vec<fn(VkQueueFamily) -> bool>) -> Device {
        todo!();
    }
}
