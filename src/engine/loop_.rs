use crate::vulkan::{
    window::{
        WindowEvent
    },
    rendering::{
        MAX_FRAMES_IN_FLIGHT,
        VkSubmitInfo,
        VkPresentInfoKHR,
        vkResetFences,
        vkQueueSubmit,
        vkWaitForFences,
        vkQueuePresentKHR
    },
    swapchain::{
        vkAcquireNextImageKHR
    }
};


impl super::Engine { pub fn start_loop(&mut self) {
    while true {
        let events = self.window.get_events();

        for event in events {match event {
            WindowEvent::Death => return,
            WindowEvent::KeyDown(keycode) => panic!(),
            _ => {}
        }}


        let image_available_semaphore = self.image_available_semaphores[self.current_frame];
        let render_finished_semaphore = self.render_finished_semaphores[self.current_frame];
        
        let in_flight_fence = self.in_flight_fences[self.current_frame];

        
        let wait_fences = [in_flight_fence];
        
        unsafe {
            vkWaitForFences(self.device, wait_fences.len() as u32, wait_fences.as_ptr(), 1, std::u64::MAX);

            vkResetFences(self.device, wait_fences.len() as u32, wait_fences.as_ptr());
        };

        
        let mut image_index: u32 = 0;

        unsafe { vkAcquireNextImageKHR(
            self.device,
            self.swapchain,
            std::u64::MAX,
            image_available_semaphore,
            0,
            &mut image_index
        )};
    
        let wait_semaphores = [image_available_semaphore];
        let signal_semaphores = [render_finished_semaphore];

        let wait_stages = [0x00000400];
        let command_buffers = [self.command_buffers[image_index as usize]];

        let submit_info = VkSubmitInfo {
            s_type: 4,
            p_next: std::ptr::null(),
            wait_sephamore_count: wait_semaphores.len() as u32,
            wait_sephamores: wait_semaphores.as_ptr(),
            wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: command_buffers.len() as u32,
            command_buffers: command_buffers.as_ptr(),
            signal_sephamore_count: signal_semaphores.len() as u32,
            signal_sephamores: signal_semaphores.as_ptr()
        };

        let submit_infos = [submit_info];

        unsafe {vkQueueSubmit(self.graphics_queue, submit_infos.len() as u32, submit_infos.as_ptr(), in_flight_fence)};


        let swapchains = [self.swapchain];
        
        let image_indices = [image_index];

        let present_info = VkPresentInfoKHR {
            s_type: 1000001001,
            p_next: std::ptr::null(),
            wait_semaphore_count: signal_semaphores.len() as u32,
            wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: swapchains.len() as u32,
            swapchains: swapchains.as_ptr(),
            image_indices: image_indices.as_ptr(),
            results: std::ptr::null_mut()
        };

        unsafe {vkQueuePresentKHR(self.presentation_queue, &present_info as *const VkPresentInfoKHR)};


        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT as usize;
    }
}}
