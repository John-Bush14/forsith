use crate::vulkan::{
    instance::{
        VkInstance,
        VkApplicationInfo,
        VkLayerProperties,
        VkExtensionProperties,
        VkInstanceCreateInfo,
        VkDebugUtilsMessengerCreateInfoEXT,
        VkDebugUtilsMessengerCallbackDataEXT,
        vkCreateInstance,
        vkCreateDebugUtilsMessengerEXT,
        vkEnumerateInstanceLayerProperties,
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

unsafe extern "system" fn vulkan_debug_callback(
    severity: u32,
    type_: u32,
    data: *const VkDebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> u32 {
    match severity {
        0x00000001 => {println!("\x1b[34m [DEBUG] {} - {:?}", type_, data)}, // debug
        0x00000010 => {println!("\x1b[32m [INFO] {} - {:?}", type_, data)}, // information
        0x00000100 => {println!("\x1b[33m [WARN] {} - {:?}", type_, data)}, // warning
        0x00000100 => {println!("\x1b[33m [PERFORMANCE] {} - {:?}", type_, data)}, // performance warning
        _ => {println!("\x1b[31m [ERROR] {} - {:?}", type_, data)} // error
    };

    print!("\x1b[0m");

    return 0;
}

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
        "VK_EXT_debug_utils",
        "VK_KHR_wayland_surface",
        "VK_KHR_win32_surface",
        "VK_EXT_metal_surface",
    );
    
    let mut supported_layers = vk_enumerate_to_vec!(vkEnumerateInstanceLayerProperties, VkLayerProperties,);

    let (layers, layer_len) = prepare_extensions![supported_layers,
        "VK_LAYER_KHRONOS_validation",
    ];

    let instance_create_info = VkInstanceCreateInfo {
        s_type: 1,
        p_next: std::ptr::null(),
        flags: 0,
        p_application_info: &application_info,
        enabled_layer_count: layer_len,
        pp_enabled_layer_names: layers,
        enabled_extension_count: extensions_len as u32,
        pp_enabled_extension_names: extensions
    };

    let mut instance = 0;

    let result = vkCreateInstance(&instance_create_info, std::ptr::null(), &mut instance);

    let debug_report_callback_create_info = VkDebugUtilsMessengerCreateInfoEXT {
        s_type: 1000128004,
        p_next: std::ptr::null(),
        flags: 0,
        severity: 0x00000001 | 0x00000010 | 0x00000100 | 0x00001000,
        type_: 0x00000001 | 0x00000002 | 0x00000004 | 0x00000008,
        fn_callback: vulkan_debug_callback,
        user_data: std::ptr::null()
    };


    if result == 0 {self.instance = instance; return;}

    panic!("InstanceCreation failed, error code: {}", result);
};}}
