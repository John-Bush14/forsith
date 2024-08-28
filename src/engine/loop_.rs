use crate::vulkan::{
    window::WindowEvent,
    rendering::{
        MAX_FRAMES_IN_FLIGHT,
        VkSubmitInfo,
        VkPresentInfoKHR,
        vkResetFences,
        vkQueueSubmit,
        vkWaitForFences,
        vkQueuePresentKHR
    },
    swapchain::vkAcquireNextImageKHR
};


impl super::Engine { pub(crate) fn process_events(&mut self) -> bool {
    let events = self.window.poll_events(self.dimensions);

    for event in &events {match *event {
        WindowEvent::Death => return true,
        WindowEvent::WindowResize(dimensions) => {self.new_dimensions = Some(dimensions)},
        _ => {}
    }};

    self.events = events;

    return false;
}}

impl super::Engine { pub(crate) fn start_loop<T>(mut self, event_loop: fn(&mut super::Engine, &mut T, f32), mut user_data: T) {
    if self.vertices.len() > 0 {self.create_vertex_buffer()}


    self.create_needed_pipelines(false);


    self.create_command_buffers();

    self.record_and_enter_command_buffers();


    self.create_sync_objects();


    let mut deltadur = std::time::Instant::now();


    loop {
        let start = std::time::Instant::now();

        if self.process_events() {return}

        let deltatime = deltadur.elapsed();
        let seconds = deltatime.as_secs() as f32;
        let nanoseconds = deltatime.subsec_nanos() as f32;
        let delta = seconds + nanoseconds / 1_000_000_000.0;

        deltadur = std::time::Instant::now();

        self.create_needed_pipelines(false);
        self.free_pipelines();

        event_loop(&mut self, &mut user_data, delta);

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

        let aspect = self.swapchain_extent.width as f32 / self.swapchain_extent.height as f32;

        self.world_view.update(aspect, self.device);

        for drawable in &mut self.drawables {
            drawable.update(image_index as usize, self.device, &self.pipelines[drawable.get_pipeline_id()]);
        }

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

        let result = unsafe {vkQueuePresentKHR(self.presentation_queue, &present_info as *const VkPresentInfoKHR)};

        if  result == 1000001003
            || result == 1000001004
            || self.new_dimensions.is_some()

            {self.recreate_swapchain()}

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT as usize;


        if self.target_fps > 0.0 {
            let elapsed_time = start.elapsed();
            let target_spf = 1.0/self.target_fps;
            let nanos = (target_spf * 1_000_000_000.0) as u64;
            let target_fps_duration = std::time::Duration::new(nanos / 1_000_000_000, (nanos % 1_000_000_000) as u32);

            if target_fps_duration > elapsed_time {
                let extra_wait_time = target_fps_duration - elapsed_time;

                std::thread::sleep(extra_wait_time);
            }
        }
    }
}}
