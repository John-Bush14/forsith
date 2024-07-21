use crate::vulkan::{
    window::{
        Window,
        x11::{
            XWindow
        }
    },
    instance::{
        VkExtensionProperties
    }
};


mod x11;

impl super::super::Engine { 
    pub fn create_test_connections(&self, supported_instance_extensions: Vec<VkExtensionProperties>) -> Vec<Box<dyn Window>> { unsafe {
        return vec![
            Box::new(XWindow::init_connection(self.dimensions))
        ];
    }}
}

impl super::super::Engine { pub fn finalize_connection(&mut self, mut connection: Box<dyn Window>, name: String) { unsafe {
    connection.init_window(name);
    self.window = connection;
}}}

impl super::super::Engine { pub fn create_surface_KHR(&mut self, instance: crate::vulkan::instance::VkInstance) { unsafe {
    self.surface_khr = self.window.create_surfaceKHR(instance);
}}}
