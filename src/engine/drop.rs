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
    }
};

impl Drop for super::Engine {
    fn drop(&mut self) {
        unsafe {
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
