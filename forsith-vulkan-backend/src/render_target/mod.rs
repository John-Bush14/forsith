use bindings::{instance::{self, VkInstance}, surface::VkSurfaceKHR};


pub mod windows;

pub(crate) mod headless;


pub trait RenderTarget {
    fn get_surface_khr(&mut self, instance: VkInstance) -> VkSurfaceKHR;

    fn drop(&self) {}
}
