pub use super::{CString, c_void, c_char, vk_make_version, VkStructureType};

pub type VkInstance = u64;

mod device;
pub use device::{
    VkDevice,
    VkDeviceCreateInfo,
    VkDeviceQueueCreateInfo,
    VkQueueFamilyProperties,
    VK_CREATE_DEVICE_CSTRING,
    VK_GET_PHYSICAL_DEVICE_QUEUE_FAMILY_PROPERTIES_CSTRING
};

mod instance;
pub use instance::{
    VkApplicationInfo,
    VkInstanceCreateInfo,
    VK_CREATE_INSTANCE_CSTRING
};

mod physical_device;
pub use physical_device::{
    VkPhysicalDevice,
    VkPhysicalDeviceType,
    VkPhysicalDeviceLimits,
    VkPhysicalDeviceProperties,
    VkPhysicalDeviceSparseProperties,
};

pub fn CreateInstance(name: String, version: u32) -> VkInstance { unsafe {
    let app_name = CString::new(name).unwrap();
    let engine_name = CString::new("Forsith").unwrap();

    let application_info = VkApplicationInfo {
        s_type: 0,
        p_next: std::ptr::null(),
        p_application_name: app_name.as_ptr(),
        application_version: version,
        p_engine_name: engine_name.as_ptr(),
        engine_version: vk_make_version(0, 1, 0),
        api_version: 0
    };

    let instance_create_info = VkInstanceCreateInfo {
        s_type: 1,
        p_next: std::ptr::null(),
        flags: 0,
        p_application_info: &application_info,
        enabled_layer_count: 0,
        pp_enabled_layer_names: std::ptr::null(),
        enabled_extension_count: 0,
        pp_enabled_extension_names: std::ptr::null()
    };

    let mut instance = 0;

    let vulkan_lib = super::get_lib();
    
    let vk_create_instance: extern "C" fn(
        *const VkInstanceCreateInfo, *const c_void, *mut VkInstance
    ) -> super::VkResult 

     = std::mem::transmute(super::dlsym(
        vulkan_lib,
        VK_CREATE_INSTANCE_CSTRING().as_ptr()
    ));

    let result = vk_create_instance(&instance_create_info, std::ptr::null(), &mut instance);

    if result == 0 {return instance}

    panic!("InstanceCreation failed, error code: {}", result);
};}




pub fn CreateDevice() -> VkPhysicalDevice { unsafe {
    let instance = super::get_instance();
    let vulkan_lib = super::get_lib();


    let vk_enumerate_physical_devices_cstring = CString::new("vkEnumeratePhysicalDevices").unwrap();

    let vk_enumerate_physical_devices: extern "C" fn(
        VkInstance, *mut u32, *mut VkPhysicalDevice
    ) -> super::VkResult = std::mem::transmute(super::load_vulkan_function(vk_enumerate_physical_devices_cstring));


    let mut physical_device_count: u32 = 0;
    let mut physical_devices: Vec<VkPhysicalDevice> = Vec::new();

    let result = vk_enumerate_physical_devices(instance, &mut physical_device_count, std::ptr::null_mut());
    if result != 0 || physical_device_count == 0 {
        panic!("Failed to enumerate physical devices");
    }   

    physical_devices.resize(physical_device_count as usize, 0);
    let result = vk_enumerate_physical_devices(instance, &mut physical_device_count, physical_devices.as_mut_ptr());
    if result != 0 {
        panic!("Failed to enumerate physical devices");
    }
    
    
    let vk_get_physical_device_properties_cstring = CString::new("vkGetPhysicalDeviceProperties").unwrap();

    let vk_get_physical_device_properties: extern "C" fn(
        VkPhysicalDevice, &mut VkPhysicalDeviceProperties
    ) -> super::VkResult = std::mem::transmute(super::load_vulkan_function(vk_get_physical_device_properties_cstring));


    let mut best_score = 0; let mut best_physical_device: Option<VkPhysicalDevice> = None;
    
    for &device in &physical_devices {
        let mut properties = std::mem::zeroed();
        let mut score = 0;

        vk_get_physical_device_properties(device, &mut properties);
        
        match properties.device_type {
            1 => {score += 1000;},
            2 => {score += 500;},
            3 => {score += 200;},
            4 => {score += 250;},
            _ => {}
        }

        if score > best_score {
            best_score = score;
            best_physical_device = Some(device);
        }
    }


    let vk_get_physical_device_queue_family_properties: extern "C" fn(
        VkPhysicalDevice, *mut u32, *mut VkQueueFamilyProperties
    ) -> c_void = std::mem::transmute(super::load_vulkan_function(VK_GET_PHYSICAL_DEVICE_QUEUE_FAMILY_PROPERTIES_CSTRING()));

    let mut device_queue_family_properties_len: u32 = 0;

    vk_get_physical_device_queue_family_properties(
        best_physical_device.unwrap(), &mut device_queue_family_properties_len, std::ptr::null_mut()
    );
    
    let mut device_queue_family_properties: Vec<VkQueueFamilyProperties> = Vec::with_capacity(device_queue_family_properties_len as usize);
    device_queue_family_properties.set_len(device_queue_family_properties_len as usize);

    vk_get_physical_device_queue_family_properties(
        best_physical_device.unwrap(), &mut device_queue_family_properties_len, device_queue_family_properties.as_mut_ptr()
    );

    let vk_create_device: extern "C" fn(
        VkPhysicalDevice, *const VkDeviceCreateInfo, *const c_void, *mut VkDevice
    ) -> super::VkResult = std::mem::transmute(super::load_vulkan_function(VK_CREATE_DEVICE_CSTRING()));
    
    let queue_priorities = [1.0f32];

    let device_queue_create_info = VkDeviceQueueCreateInfo {
        s_type: 2,
        p_next: std::ptr::null(),
        flags: 0,
        queue_family_index: 0,
        queue_count: 0,
        queue_priorities: queue_priorities.as_ptr()
    };

    let device_create_info = VkDeviceCreateInfo {
        s_type: 3,
        p_next: std::ptr::null(),
        flags: 0,
        queue_create_info_count: 1,
        queue_create_infos: &device_queue_create_info,
        enabled_layer_count: 0,
        enabled_layer_names: std::ptr::null(),
        enabled_extension_count: 0,
        enabled_extension_names: std::ptr::null(),
        enabled_features: std::ptr::null()
    };
    
    let mut device: VkDevice = 0;

    let result = vk_create_device(
        best_physical_device.expect("No physical devices found!"), &device_create_info, std::ptr::null(), &mut device
    );

    if result == 0 {return device;}
    
    panic!("vkCreateDevice failed!");
}}
