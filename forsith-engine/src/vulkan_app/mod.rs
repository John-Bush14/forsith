use bindings::{command_pool::VkCommandPool, device::VkDevice, instance::VkInstance, physical_device::{VkPhysicalDevice, VkQueue, VkQueueFamily}};


mod creation;

mod drop;

pub(crate) mod device;

#[allow(dead_code)]
pub struct VulkanApp {
    instance: VkInstance,
    general_device: device::Device,
    transient_command_pool: VkCommandPool,
}
