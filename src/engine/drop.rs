use crate::vulkan::{
    instance::{
        vkDestroyInstance
    },
    devices::{
        device::{
            vkDestroyDevice
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
    }
};

impl Drop for super::Engine {
    fn drop(&mut self) {
        unsafe {
            self.image_available_semaphores.iter().chain(self.render_finished_semaphores.iter())
                .for_each(|&semaphore| vkDestroySemaphore(self.device, semaphore, std::ptr::null()));
            
            self.in_flight_fences.iter().for_each(|&fence| vkDestroyFence(self.device, fence, std::ptr::null()));

            vkDestroyCommandPool(self.device, self.command_pool, std::ptr::null());

            self.framebuffers.iter().for_each(|&framebuffer| vkDestroyFramebuffer(self.device, framebuffer, std::ptr::null()));

            self.shader_modules.iter().for_each(|&shader_module| vkDestroyShaderModule(self.device, shader_module, std::ptr::null()));

            vkDestroyRenderPass(self.device, self.render_pass, std::ptr::null());

            vkDestroyPipelineLayout(self.device, self.pipeline_layout, std::ptr::null());

            vkDestroyPipeline(self.device, self.pipeline, std::ptr::null());

            self.swapchain_image_views.iter().for_each(|&image_view| vkDestroyImageView(self.device, image_view, std::ptr::null()));

            vkDestroySwapchainKHR(self.device, self.swapchain, std::ptr::null());

            vkDestroySurfaceKHR(self.instance, self.surface_khr, std::ptr::null());

            self.window.commit_suicide();

            vkDestroyDevice(self.device, std::ptr::null());

            vkDestroyInstance(self.instance, std::ptr::null());
        };
    }
}
