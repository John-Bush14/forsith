use crate::vulkan::{
    devices::{
        physical_device::{
            VkPhysicalDevice,
            VkQueueFamilyProperties,
            vkEnumeratePhysicalDevices,
            vkGetPhysicalDeviceProperties,
            vkGetPhysicalDeviceQueueFamilyProperties,
            vkGetPhysicalDeviceSurfaceSupportKHR
        },
        device::{
            VkDevice,
            VkDeviceCreateInfo,
            VkDeviceQueueCreateInfo,
            vkCreateDevice
        }
    },
    window::{
        VkSurfaceKHR
    },
    VkBool32
};

fn get_physical_device_queue_properties(physical_device: VkPhysicalDevice) -> Vec<VkQueueFamilyProperties> { unsafe {
    let mut device_queue_family_properties_len: u32 = 0;

    vkGetPhysicalDeviceQueueFamilyProperties(
        physical_device, &mut device_queue_family_properties_len, std::ptr::null_mut()
    );

    let mut device_queue_family_properties: Vec<VkQueueFamilyProperties> = Vec::with_capacity(device_queue_family_properties_len as usize);
    device_queue_family_properties.set_len(device_queue_family_properties_len as usize);

    vkGetPhysicalDeviceQueueFamilyProperties(
        physical_device, &mut device_queue_family_properties_len, device_queue_family_properties.as_mut_ptr()
    );
    
    return device_queue_family_properties;
}}

impl super::super::Engine { pub fn create_device(&mut self) { unsafe {
    let instance = self.instance;

    let mut physical_device_count: u32 = 0;
    let mut physical_devices: Vec<VkPhysicalDevice> = Vec::new();

    let result = vkEnumeratePhysicalDevices(instance, &mut physical_device_count, std::ptr::null_mut());
    if result != 0 || physical_device_count == 0 {
        panic!("Failed to enumerate physical devices");
    }   

    physical_devices.resize(physical_device_count as usize, 0);
    let result = vkEnumeratePhysicalDevices(instance, &mut physical_device_count, physical_devices.as_mut_ptr());
    if result != 0 {
        panic!("Failed to enumerate physical devices");
    }
    
    let mut best_score = 0;
    
    let (&best_physical_device, graphics_queue, presentation_queue) = physical_devices
        .iter() 
        .map(|device| {
            let family_queue_properties = get_physical_device_queue_properties(device.clone());
            
            let graphics_queue = family_queue_properties.iter().position(|x| x.flags & 0x00000001 != 0);
            
            let mut presentation_queue: Option<usize> = None;
            
            for i in 0..family_queue_properties.len() {
                let mut queue_supports_KHR: VkBool32 = 0;
                vkGetPhysicalDeviceSurfaceSupportKHR(device.clone(), i as u32, self.surface_khr.clone(), &mut queue_supports_KHR); 
                if queue_supports_KHR == 1 {presentation_queue = Some(i); break;}
            }

            println!("graph {:?}, present {:?}", graphics_queue, presentation_queue);

            return (device, graphics_queue, presentation_queue)
        })
        .filter(|(_, graphics_queue, presentation_queue)| return graphics_queue.is_some() && presentation_queue.is_some())
        .map(|(device, graphics_queue, presentation_queue)| return (device, graphics_queue.unwrap() as u32, presentation_queue.unwrap() as u32))
        .max_by_key(|(&device, _, _)| {
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

    let device_create_info = VkDeviceCreateInfo {
        s_type: 3,
        p_next: std::ptr::null(),
        flags: 0,
        queue_create_info_count: device_queue_create_infos.len() as u32,
        queue_create_infos: device_queue_create_infos.as_ptr(),
        enabled_layer_count: 0,
        enabled_layer_names: std::ptr::null(),
        enabled_extension_count: 0,
        enabled_extension_names: std::ptr::null(),
        enabled_features: std::ptr::null()
    };
    
    let mut device: VkDevice = 0;

    let result = vkCreateDevice(
        best_physical_device, &device_create_info, std::ptr::null(), &mut device
    );

    if result == 0 {self.device = device; return;}
    
    panic!("vkCreateDevice failed!");
}}}
