use crate::vulkan::{
    instance::{
        VkInstance,
        VkInstanceCreateInfo,
        VkApplicationInfo,
        VkExtensionProperties,
        vkCreateInstance,
        vkEnumerateInstanceExtensionProperties
    },
    VkResult,
    vk_make_version,
};

use crate::{
    vk_enumerate_to_vec,
    prepare_extensions
};

use std::ffi::{
    c_void,
    CString,
    c_char,
    CStr
};

impl crate::engine::Engine { pub fn create_instance(&mut self, supported_extensions: Vec<VkExtensionProperties>) { unsafe {
    let app_name = CString::new(self.app_name.clone()).unwrap();
    let engine_name = CString::new("Forsith").unwrap();

    let application_info = VkApplicationInfo {
        s_type: 0,
        p_next: std::ptr::null(),
        p_application_name: app_name.as_ptr(),
        application_version: self.app_version,
        p_engine_name: engine_name.as_ptr(),
        engine_version: vk_make_version(0, 1, 0),
        api_version: vk_make_version(1, 1, 0)
    };
    
    let (extensions, extensions_len) = prepare_extensions!(supported_extensions,
        "VK_KHR_surface",
        "VK_KHR_xlib_surface",
    );

    let instance_create_info = VkInstanceCreateInfo {
        s_type: 1,
        p_next: std::ptr::null(),
        flags: 0,
        p_application_info: &application_info,
        enabled_layer_count: 0,
        pp_enabled_layer_names: std::ptr::null(),
        enabled_extension_count: extensions_len as u32,
        pp_enabled_extension_names: extensions
    };

    let mut instance = 0;

    let result = vkCreateInstance(&instance_create_info, std::ptr::null(), &mut instance);

    if result == 0 {self.instance = instance; return;}

    panic!("InstanceCreation failed, error code: {}", result);
};}}
