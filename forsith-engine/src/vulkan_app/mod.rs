use bindings::{command_pool::VkCommandPool, device::VkDevice, instance::VkInstance, physical_device::{VkPhysicalDevice, VkQueue, VkQueueFamily}};


mod creation;

mod drop;


#[allow(dead_code)]
pub struct VulkanApp {
    instance: VkInstance,
    device: VkDevice,
    physical_device: VkPhysicalDevice,
    transient_command_pool: VkCommandPool,
    graphics_queue_family: VkQueueFamily,
    graphics_queue: VkQueue
}
