use bindings::command_pool::{VkCommandBuffer, VkCommandPool, VkCommandPoolCreateFlags};


#[allow(dead_code)]
pub struct CommandPool {
    command_pool: VkCommandPool,
    command_buffers: Vec<VkCommandBuffer>,
    flags: VkCommandPoolCreateFlags
}
