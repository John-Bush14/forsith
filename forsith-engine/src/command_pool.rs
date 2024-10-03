use bindings::{command_pool::{vk_create_command_pool, VkCommandBuffer, VkCommandPool, VkCommandPoolCreateFlags, VkCommandPoolCreateInfo}, device::VkDevice, physical_device::VkQueueFamily};


#[allow(dead_code)]
pub struct CommandPool {
    command_pool: VkCommandPool,
    command_buffers: Vec<VkCommandBuffer>,
    flags: VkCommandPoolCreateFlags,
    queue_family: VkQueueFamily
}


impl CommandPool {
    pub fn new(device: VkDevice, flags: VkCommandPoolCreateFlags, queue_family: VkQueueFamily) -> Self  {
        let create_info = VkCommandPoolCreateInfo {
            s_type: VkCommandPoolCreateInfo::structure_type(),
            p_next: std::ptr::null(),
            flags: flags.clone(),
            queue_family
        };


        let mut vk_command_pool = 0;

        vk_create_command_pool(device, &create_info as *const VkCommandPoolCreateInfo, std::ptr::null(), &mut vk_command_pool);


        let command_pool = CommandPool {
            command_pool: vk_command_pool,
            command_buffers: vec!(),
            flags,
            queue_family
        };

        return command_pool;
    }
}
