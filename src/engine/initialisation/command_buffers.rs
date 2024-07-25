use crate::vulkan::{
    commands::{
        command_pool::{
            VkCommandPool,
            VkCommandPoolCreateInfo,
            vkCreateCommandPool
        },
        command_buffer::{
            VkCommandBuffer,
            VkCommandBufferBeginInfo,
            VkCommandBufferAllocateInfo,
            vkEndCommandBuffer,
            vkBeginCommandBuffer,
            vkAllocateCommandBuffers
        },
        VkClearValue,
        VkClearColorValue,
        VkRenderPassBeginInfo,
        vkCmdDrawIndexed,
        vkCmdBindPipeline,
        vkCmdEndRenderPass,
        vkCmdBeginRenderPass,
        vkCmdBindIndexBuffer,
        vkCmdBindVertexBuffers,
        vkCmdBindDescriptorSets
    },
    pipeline::{
        VkRect2D,
        VkOffset2D,
        VkPipelineLayout
    },
    vertex::{
        INDICES
    },
    uniform::{
        VkDescriptorSet
    }
};


impl super::super::Engine { pub fn create_command_pool(&mut self, transient: bool) { unsafe {
    let command_pool_create_info = VkCommandPoolCreateInfo {
        s_type: 39,
        p_next: std::ptr::null(),
        flags: {if transient {0x00000001} else {0}},
        queue_family_index: self.graphics_family
    };

    vkCreateCommandPool(
        self.device,
        &command_pool_create_info as *const VkCommandPoolCreateInfo,
        std::ptr::null(),
        {if transient {&mut self.transient_command_pool} else {&mut self.command_pool}}
    );
}}}


impl super::super::Engine { pub fn create_command_buffers(&mut self) { unsafe {
    self.command_buffers = vec!();

    let allocate_info = VkCommandBufferAllocateInfo {
        s_type: 40,
        p_next: std::ptr::null(),
        command_pool: self.command_pool,
        level: 0,
        command_buffer_count: self.framebuffers.len() as u32
    };

    let mut command_buffers: Vec<VkCommandBuffer> = self.framebuffers.iter().map(|_| 0).collect();


    vkAllocateCommandBuffers(
        self.device, 
        &allocate_info as *const VkCommandBufferAllocateInfo,
        command_buffers.as_mut_ptr()
    );

    command_buffers.iter().enumerate()
        .for_each(|(i, &command_buffer)| {
            let framebuffer = self.framebuffers[i];

            let command_buffer_begin_info = VkCommandBufferBeginInfo {
                s_type: 42,
                p_next: std::ptr::null(),
                flags: 0x00000004,
                inheritance_info: std::ptr::null()
            };

            vkBeginCommandBuffer(command_buffer, &command_buffer_begin_info as *const VkCommandBufferBeginInfo);


            let clear_values = [VkClearValue {color: VkClearColorValue {float32: [0.0, 0.0, 0.0, 1.0]}}];
            
            let render_pass_begin_info = VkRenderPassBeginInfo {
                s_type: 43,
                p_next: std::ptr::null(),
                render_pass: self.render_pass,
                framebuffer: framebuffer,
                render_area: VkRect2D {offset: VkOffset2D {x: 0, y: 0}, extent: self.swapchain_extent.clone()},
                clear_value_count: clear_values.len() as u32,
                clear_values: clear_values.as_ptr()
            };

            vkCmdBeginRenderPass(command_buffer, &render_pass_begin_info as *const VkRenderPassBeginInfo, 0);


            vkCmdBindPipeline(command_buffer, 0, self.pipeline);

            
            let vertex_buffers = [self.vertex_buffer];

            let offsets = [0];

            if vertex_buffers[0] != 0 {vkCmdBindVertexBuffers(
                command_buffer, 
                0,
                vertex_buffers.len() as u32,
                vertex_buffers.as_ptr(),
                offsets.as_ptr()
            )};

            if self.index_buffer != 0 {vkCmdBindIndexBuffer(
                command_buffer,
                self.index_buffer,
                0,
                0
            );}

            vkCmdBindDescriptorSets(
                command_buffer,
                0,
                self.pipeline_layout,
                0,
                1,
                &self.descriptor_sets[i],
                0,
                std::ptr::null()
            );

            if self.index_buffer != 0 {vkCmdDrawIndexed(command_buffer, self.indices.len() as u32, 1, 0, 0, 0)}

            
            vkCmdEndRenderPass(command_buffer);

            vkEndCommandBuffer(command_buffer);
        });
    
    self.command_buffers = command_buffers;
}}}
