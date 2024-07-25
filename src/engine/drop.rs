use crate::vulkan::{
    instance::{
        vkDestroyInstance
    },
    devices::{
        device::{
            vkDestroyDevice,
            vkDeviceWaitIdle
        }
    },
    window::{
        vkDestroySurfaceKHR
    },
    swapchain::{
        vkDestroySwapchainKHR,
        image_view::{
            vkDestroyImageView
        }
    },
    pipeline::{
        vkDestroyShaderModule,
        vkDestroyPipelineLayout,
        vkDestroyRenderPass,
        vkDestroyPipeline,
        vkDestroyFramebuffer
    },
    commands::{
        command_pool::{
            vkDestroyCommandPool
        },
    },
    rendering::{
        vkDestroyFence,
        vkDestroySemaphore
    },
    vertex::{
        vkFreeMemory,
        vkDestroyBuffer
    },
    uniform::{
        vkDestroyDescriptorPool,
        vkDestroyDescriptorSetLayout
    }
};

impl Drop for super::Engine {
    fn drop(&mut self) {
        unsafe {
            if self.image_available_semaphores.len() == 0 {return}

            vkDeviceWaitIdle(self.device);

            self.image_available_semaphores.iter().chain(self.render_finished_semaphores.iter())
                .for_each(|&semaphore| vkDestroySemaphore(self.device, semaphore, std::ptr::null()));
            
            self.cleanup_swapchain();
            
            self.in_flight_fences.iter().for_each(|&fence| vkDestroyFence(self.device, fence, std::ptr::null()));
            
            vkDestroyDescriptorPool(self.device, self.descriptor_pool, std::ptr::null());

            vkDestroyDescriptorSetLayout(self.device, self.descriptor_set_layout, std::ptr::null());

            self.uniform_buffers.iter().for_each(|&buffer| vkDestroyBuffer(self.device, buffer, std::ptr::null()));
            self.uniform_buffer_memories.iter().for_each(|&memory| vkFreeMemory(self.device, memory, std::ptr::null()));
            
            if self.index_buffer != 0 {
                vkDestroyBuffer(self.device, self.index_buffer, std::ptr::null());
                vkFreeMemory(self.device, self.index_buffer_memory, std::ptr::null());
            }
            
            if self.vertex_buffer != 0 {
                vkDestroyBuffer(self.device, self.vertex_buffer, std::ptr::null());
                vkFreeMemory(self.device, self.vertex_buffer_memory, std::ptr::null());
            }

            vkDestroyCommandPool(self.device, self.command_pool, std::ptr::null());
            vkDestroyCommandPool(self.device, self.transient_command_pool, std::ptr::null());

            vkDestroySurfaceKHR(self.instance, self.surface_khr, std::ptr::null());

            self.window.commit_suicide();

            vkDestroyDevice(self.device, std::ptr::null());

            vkDestroyInstance(self.instance, std::ptr::null());
        };
    }
}
