use crate::vulkan::{
    commands::command_pool::vkDestroyCommandPool, devices::device::{
            vkDestroyDevice,
            vkDeviceWaitIdle
        }, image::{vkDestroyImage, vkDestroyImageView, vkDestroySampler}, instance::vkDestroyInstance, pipeline::{
        vkDestroyPipelineLayout, vkDestroyShaderModule
    }, rendering::{
        vkDestroyFence,
        vkDestroySemaphore
    }, uniform::{
        vkDestroyDescriptorPool,
        vkDestroyDescriptorSetLayout
    }, vertex::{
        vkDestroyBuffer, vkFreeMemory
    }, window::vkDestroySurfaceKHR
};

impl Drop for crate::Drawable {
    fn drop(&mut self) { unsafe { if self.uniform_buffers.len() > 0 {
        self.uniform_buffers.iter().for_each(|vec| vec.iter().for_each(|(buffer, memory)| {
            vkDestroyBuffer(self.device, *buffer, std::ptr::null());
            vkFreeMemory(self.device, *memory, std::ptr::null());
        }));

        if let Some((_, (image, memory, image_view, sampler))) = self.image {
            vkFreeMemory(self.device, memory, std::ptr::null());
            vkDestroyImage(self.device, image, std::ptr::null());
            vkDestroyImageView(self.device, image_view, std::ptr::null());
            vkDestroySampler(self.device, sampler, std::ptr::null())
        }
        
        vkDestroyBuffer(self.device, self.indice_buffer, std::ptr::null());
        vkFreeMemory(self.device, self.indice_memory, std::ptr::null());
    }}}
}

impl Drop for super::Engine {
    fn drop(&mut self) { 
        unsafe {
            if self.image_available_semaphores.len() == 0 {return}

            vkDeviceWaitIdle(self.device);

            while let Some(drawable) = self.drawables.pop() {
                std::mem::drop(drawable);
            }

            self.image_available_semaphores.iter().chain(self.render_finished_semaphores.iter())
                .for_each(|&semaphore| vkDestroySemaphore(self.device, semaphore, std::ptr::null()));
            
            self.pipelines.iter().for_each(|pipeline| {
                vkDestroyShaderModule(self.device, pipeline.vertex_shader, std::ptr::null());
                vkDestroyShaderModule(self.device, pipeline.fragment_shader, std::ptr::null());
            });

            self.cleanup_swapchain();
            
            self.in_flight_fences.iter().for_each(|&fence| vkDestroyFence(self.device, fence, std::ptr::null()));
            

            vkDestroyDescriptorPool(self.device, self.descriptor_pool, std::ptr::null());

            self.pipeline_layouts.clone().into_iter().for_each(|(_, (pipeline_layout, descriptor_set_layout))| {
                vkDestroyPipelineLayout(self.device, pipeline_layout, std::ptr::null());
                vkDestroyDescriptorSetLayout(self.device, descriptor_set_layout, std::ptr::null());
            });
            
            if self.vertex_buffer != 0 {
                vkDestroyBuffer(self.device, self.vertex_buffer, std::ptr::null());
                vkFreeMemory(self.device, self.vertex_buffer_memory, std::ptr::null());
            }
    
            self.world_view.get_2d_uniform_buffers().iter().chain(self.world_view.get_3d_uniform_buffers()).for_each(
                |(buffer, memory)| {
                    vkDestroyBuffer(self.device, *buffer, std::ptr::null()); 
                    vkFreeMemory(self.device, *memory, std::ptr::null());
                }
            );

            vkDestroyCommandPool(self.device, self.command_pool, std::ptr::null());
            vkDestroyCommandPool(self.device, self.transient_command_pool, std::ptr::null());

            vkDestroySurfaceKHR(self.instance, self.surface_khr, std::ptr::null());

            self.window.commit_suicide();

            vkDestroyDevice(self.device, std::ptr::null());

            vkDestroyInstance(self.instance, std::ptr::null());
        };
    }
}
