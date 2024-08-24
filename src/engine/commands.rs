use crate::vulkan::{commands::{command_buffer::{vkAllocateCommandBuffers, vkBeginCommandBuffer, vkEndCommandBuffer, vkFreeCommandBuffers, VkCommandBuffer, VkCommandBufferAllocateInfo, VkCommandBufferBeginInfo}, command_pool::VkCommandPool}, devices::device::VkQueue, rendering::{vkQueueSubmit, vkQueueWaitIdle, VkSubmitInfo}};

impl crate::engine::Engine {pub(crate) fn execute_one_time_command<F: FnOnce(VkCommandBuffer)>(
    &self,
    command_pool: VkCommandPool,
    queue: VkQueue,
    executor: F,
) {
    let allocation_info = VkCommandBufferAllocateInfo {
        s_type: 40,
        p_next: std::ptr::null(),
        command_pool,
        level: 0,
        command_buffer_count: 1
    };

    let mut command_buffers = [0];

    unsafe {vkAllocateCommandBuffers(self.device, &allocation_info as *const VkCommandBufferAllocateInfo, command_buffers.as_mut_ptr())};

    let command_buffer = command_buffers[0];


    let begin_info = VkCommandBufferBeginInfo {
        s_type: 42,
        p_next: std::ptr::null(),
        flags: 0x00000001,
        inheritance_info: std::ptr::null()
    };

    unsafe {vkBeginCommandBuffer(command_buffer, &begin_info as *const VkCommandBufferBeginInfo)};

    executor(command_buffer);

    unsafe {vkEndCommandBuffer(command_buffer)};


    let mut submit_info: VkSubmitInfo = unsafe {std::mem::zeroed()};

    submit_info.s_type = 4;

    submit_info.command_buffer_count = command_buffers.len() as u32;
    submit_info.command_buffers = command_buffers.as_ptr();

    let submit_infos = [submit_info];

    unsafe {vkQueueSubmit(queue, submit_infos.len() as u32, submit_infos.as_ptr(), 0)};


    unsafe {vkQueueWaitIdle(queue)};


    unsafe {vkFreeCommandBuffers(self.device, command_pool, command_buffers.len() as u32, command_buffers.as_ptr())};
}}
