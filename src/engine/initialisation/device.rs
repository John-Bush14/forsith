use crate::vulkan::{
    devices::{
        physical_device::{
            VkPhysicalDevice,
            VkExtensionProperties,
            VkQueueFamilyProperties,
            vkEnumeratePhysicalDevices,
            vkGetPhysicalDeviceProperties,
            vkGetPhysicalDeviceQueueFamilyProperties,
            vkGetPhysicalDeviceSurfaceSupportKHR,
            vkEnumerateDeviceExtensionProperties
        },
        device::{
            VkDevice,
            VkDeviceCreateInfo,
            VkDeviceQueueCreateInfo,
            vkCreateDevice
        }
    },
    window::{
        VkSurfaceKHR,
        Window
    },
    VkBool32
};

use crate::{
    vk_enumerate_to_vec,
    prepare_extensions
};

use std::ffi::{
    c_char,
    CString
};


impl super::super::Engine { pub fn create_device(&mut self, mut test_window_connections: Vec<Box<dyn Window>>) -> (Box<dyn Window>, u32, u32) { unsafe {
    let instance = self.instance;

    let mut physical_device_count: u32 = 0;
    let mut physical_devices: Vec<VkPhysicalDevice> = vk_enumerate_to_vec!(
        vkEnumeratePhysicalDevices,
        VkPhysicalDevice,
        instance,
    );

    let mut best_score = 0;
    
    let (&best_physical_device, graphics_queue, presentation_queue, chosen_window_connection) = physical_devices
        .iter() 
        .map(|device| {
            let family_queue_properties = vk_enumerate_to_vec!(vkGetPhysicalDeviceQueueFamilyProperties, VkQueueFamilyProperties, device.clone(),);
            
            let graphics_queue = family_queue_properties.iter().position(|x| x.flags & 0x00000001 != 0);
            
            let mut presentation_queue: Option<usize> = None;
            let mut window_connection: Option<usize> = None;
            
            for i in 0..family_queue_properties.len() { 
                for connection in 0..test_window_connections.len() {
                    if test_window_connections[connection].supports_physical_device_queue(device.clone(), i as u32) {
                        presentation_queue = Some(i); 
                        window_connection = Some(connection); 
                        break;
                    }
                }
            }

            println!("graph {:?}, present {:?}", graphics_queue, presentation_queue);

            return (device, graphics_queue, presentation_queue, window_connection)
        })
        .filter(|(_, graphics_queue, presentation_queue, window_connection)| 
            return graphics_queue.is_some() && presentation_queue.is_some() && window_connection.is_some()
        )
        .map(|(device, graphics_queue, presentation_queue, connection)| 
            return (device, graphics_queue.unwrap() as u32, presentation_queue.unwrap() as u32, connection.unwrap())
        )
        .max_by_key(|(&device, _, _, _)| {
            let mut properties = std::mem::zeroed();
            let mut score: u16 = 0;
            
            vkGetPhysicalDeviceProperties(device, &mut properties);
        
            match properties.device_type {
                1 => {score += 1000;},
                2 => {score += 500;},
                3 => {score += 250;},
                4 => {score += 125;},
                _ => {}
            }

            return score;
        }).expect("No supported physical devices!");

    self.physical_device = best_physical_device;    

    let queue_priorities = [1.0f32];

    let graphics_device_queue_create_info = VkDeviceQueueCreateInfo {
        s_type: 2,
        p_next: std::ptr::null(),
        flags: 0,
        queue_family_index: graphics_queue,
        queue_count: 1,
        queue_priorities: queue_priorities.as_ptr()
    };
    
    let presentation_device_queue_create_info = VkDeviceQueueCreateInfo {
        s_type: 2,
        p_next: std::ptr::null(),
        flags: 0,
        queue_family_index: presentation_queue,
        queue_count: 1,
        queue_priorities: queue_priorities.as_ptr()
    };

    let device_queue_create_infos = [graphics_device_queue_create_info, presentation_device_queue_create_info];
    
    let supported_extensions = vk_enumerate_to_vec!(
        vkEnumerateDeviceExtensionProperties, 
        VkExtensionProperties,
        best_physical_device, 
        std::ptr::null(),
    );

    let (extensions, extensions_len) = prepare_extensions!(supported_extensions,
        "VK_KHR_swapchain",
    );

    let device_create_info = VkDeviceCreateInfo {
        s_type: 3,
        p_next: std::ptr::null(),
        flags: 0,
        queue_create_info_count: device_queue_create_infos.len() as u32,
        queue_create_infos: device_queue_create_infos.as_ptr(),
        enabled_layer_count: 0,
        enabled_layer_names: std::ptr::null(),
        enabled_extension_count: extensions_len,
        enabled_extension_names: extensions,
        enabled_features: std::ptr::null()
    };
    
    let mut device: VkDevice = 0;

    let result = vkCreateDevice(
        best_physical_device, &device_create_info, std::ptr::null(), &mut device
    );

    if result == 0 {self.device = device; return (test_window_connections.remove(chosen_window_connection), presentation_queue, graphics_queue);}
    
    panic!("vkCreateDevice failed!");
}}}