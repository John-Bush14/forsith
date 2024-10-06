use bindings::instance::VkInstance;

use crate::command_pool::CommandPool;


pub(crate) mod creation;

mod drop;


#[allow(dead_code)]
pub struct VulkanApp {
    instance: VkInstance,
    general_device: crate::device::Device,
    transient_command_pool: CommandPool,
}
