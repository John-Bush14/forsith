use bindings::{command_pool::VkCommandPool, instance::VkInstance};


pub(crate) mod creation;

mod drop;


#[allow(dead_code)]
pub struct VulkanApp {
    instance: VkInstance,
    general_device: crate::device::Device,
    transient_command_pool: VkCommandPool,
}
