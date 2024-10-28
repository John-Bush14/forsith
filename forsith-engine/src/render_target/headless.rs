use bindings::{instance::{self, VkInstance}, surface::VkSurfaceKHR};

use super::RenderTarget;


pub struct HeadlessRenderTarget {
    surface_khr: VkSurfaceKHR
}


impl RenderTarget for HeadlessRenderTarget {
    fn get_surface_khr(&mut self, instance: VkInstance) -> VkSurfaceKHR {
        todo!();
    }

    fn drop(&self) {}
}


pub fn headless_window() -> HeadlessRenderTarget {todo!();}
