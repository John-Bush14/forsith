use bindings::{command_pool::{vk_create_command_pool, vk_destroy_command_pool, VkCommandBuffer, VkCommandPool, VkCommandPoolCreateFlags, VkCommandPoolCreateInfo}, device::VkDevice, physical_device::VkQueueFamily};

use crate::{vulkan_app::device::Device, DynError};


#[allow(dead_code)]
pub struct CommandPool {
    command_pool: VkCommandPool,
    command_buffers: Vec<VkCommandBuffer>,
    flags: VkCommandPoolCreateFlags,
    queue_family: VkQueueFamily,
    vk_device: VkDevice
}


#[allow(dead_code)]
impl CommandPool {
    pub fn destroy(&self) {
        vk_destroy_command_pool(self.vk_device, self.command_pool, std::ptr::null());
    }

    pub fn new(device: &Device, flags: VkCommandPoolCreateFlags, queue_family: VkQueueFamily) -> Result<Self, DynError>  {
        let create_info = VkCommandPoolCreateInfo {
            s_type: VkCommandPoolCreateInfo::structure_type(),
            p_next: std::ptr::null(),
            flags: flags.clone(),
            queue_family
        };


        let vk_device = *device.get_device();


        let mut vk_command_pool = 0;

        vk_create_command_pool(vk_device, &create_info as *const VkCommandPoolCreateInfo, std::ptr::null(), &mut vk_command_pool).result()?;


        let command_pool = CommandPool {
            command_pool: vk_command_pool,
            command_buffers: vec!(),
            flags,
            queue_family,
            vk_device
        };

        return Ok(command_pool);
    }
}


#[cfg(test)]
mod command_pool_tests {
    use bindings::{command_pool::VkCommandPoolCreateFlags, instance::vk_destroy_instance, vk_version};

    use crate::{vulkan_app::device::create_device, vulkan_app::creation::instance::create_instance, DynError};
    use super::CommandPool;

    #[test]
    pub fn create_command_pool_test() -> Result<(), DynError> {
        let instance = create_instance("command pool creation test", vk_version(0, 1, 0)).expect("failed because of instance creation");

        let device = create_device(instance, vec![|_, _| return true]).expect("failed because of device creation");


        let command_pool = CommandPool::new(&device, VkCommandPoolCreateFlags(0), device.get_queue(0).family())?;

        command_pool.destroy();


        device.destroy().expect("test failed because of device destroyal");

        vk_destroy_instance(instance, std::ptr::null());


        return Ok(());
    }
}
