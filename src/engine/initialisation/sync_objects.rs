use crate::vulkan::{
    rendering::{
        MAX_FRAMES_IN_FLIGHT,
        VkFence,
        VkSemaphore,
        VkFenceCreateInfo,
        VkSemaphoreCreateInfo,
        vkCreateFence,
        vkCreateSemaphore
    }
};


impl super::super::Engine { pub fn create_sync_objects(&mut self) { unsafe {
    let semaphore_create_info = VkSemaphoreCreateInfo {s_type: 9, p_next: std::ptr::null(), flags: 0};
    
    let fence_create_info = VkFenceCreateInfo {s_type: 8, p_next: std::ptr::null(), flags: 0x00000001};

    for _ in 0 .. MAX_FRAMES_IN_FLIGHT {
        let mut image_available_semaphore: VkSemaphore = 0;

        vkCreateSemaphore(
            self.device,
            &semaphore_create_info as *const VkSemaphoreCreateInfo,
            std::ptr::null(),
            &mut image_available_semaphore
        );

        self.image_available_semaphores.push(image_available_semaphore);


        let mut render_finished_semaphore: VkSemaphore = 0;

        vkCreateSemaphore(
            self.device,
            &semaphore_create_info as *const VkSemaphoreCreateInfo,
            std::ptr::null(),
            &mut render_finished_semaphore
        );

        self.render_finished_semaphores.push(render_finished_semaphore);


        let mut in_flight_fence: VkFence = 0;

        vkCreateFence(
            self.device,
            &fence_create_info as *const VkFenceCreateInfo,
            std::ptr::null(),
            &mut in_flight_fence
        );

        self.in_flight_fences.push(in_flight_fence);
    }
}}}
