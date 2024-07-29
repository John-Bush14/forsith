use crate::vulkan::{
    window::{
        Window,
    },
    instance::{
        VkExtensionProperties
    }
};

#[cfg(target_os = "linux")]
mod x11;
#[cfg(target_os = "linux")]
use crate::vulkan::window::x11::XWindow;

#[cfg(target_os = "linux")]
mod wayland;
#[cfg(target_os = "linux")]
use crate::vulkan::window::wayland::WayWindow;

#[cfg(target_os = "windows")]
mod win32;
#[cfg(target_os = "windows")]
use crate::vulkan::window::win32::WinWindow;

#[cfg(target_os = "macos")]
mod metal;
#[cfg(target_os = "macos")]
use crate::vulkan::window::metal::MWindow;

impl super::super::Engine { 
    pub fn create_test_connections(&self, supported_instance_extensions: Vec<VkExtensionProperties>) -> Vec<Box<dyn Window>> { unsafe {
        let mut test_connections: Vec<Box<dyn Window>> = vec![];

        if cfg!(target_os = "linux") {
            if std::env::var("XDG_SESSION_TYPE").expect("XDG_SESSION_TYPE not set") == "x11" {
                #[cfg(target_os = "linux")]
                test_connections.push(Box::new(XWindow::init_connection(self.dimensions)));
            } else {
                #[cfg(target_os = "linux")]
                test_connections.push(Box::new(WayWindow::init_connection(self.dimensions)));
            }
        }

        #[cfg(target_os = "windows")]
        test_connections.push(Box::new(WinWindow::init_connection(self.dimensions)));
        
        #[cfg(target_os = "macos")]
        test_connections.push(Box::new(MWindow::init_connection(self.dimensions)));

        return test_connections;
    }}
}

impl super::super::Engine { pub fn finalize_connection(&mut self, mut connection: Box<dyn Window>, name: String) { unsafe {
    connection.init_window(name);
    self.window = connection;
}}}

impl super::super::Engine { pub fn create_surface_KHR(&mut self, instance: crate::vulkan::instance::VkInstance) { unsafe {
    self.surface_khr = self.window.create_surfaceKHR(instance);
}}}
