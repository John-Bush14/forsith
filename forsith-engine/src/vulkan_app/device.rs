use bindings::{device::VkDevice, physical_device::{VkPhysicalDevice, VkQueue, VkQueueFamily, VkQueueFlagBits}};


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
