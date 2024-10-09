use bindings::instance::VkInstance;

use crate::command_pool::CommandPool;


pub(crate) mod creation;

pub(crate) mod device;

mod drop;


#[allow(dead_code)]
pub struct VulkanApp {
    instance: VkInstance,
    general_device: device::Device,
    transient_command_pool: CommandPool,
}
