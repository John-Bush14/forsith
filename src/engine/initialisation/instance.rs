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

use std::ffi::{
    c_void,
    CString,
    c_char,
    CStr
};

const EXTENSIONS: fn() -> [CString;2] = || return [
    CString::new("VK_KHR_surface").unwrap(),
    CString::new("VK_KHR_xlib_surface").unwrap(),
];

fn get_instance_extension_properties() -> Vec<VkExtensionProperties> { unsafe {
    let mut instance_extension_properties_len: u32 = 0;

    vkEnumerateInstanceExtensionProperties(
        std::ptr::null(), &mut instance_extension_properties_len, std::ptr::null_mut()
    );

    let mut instance_extension_properties: Vec<VkExtensionProperties> = Vec::with_capacity(instance_extension_properties_len as usize);
    instance_extension_properties.set_len(instance_extension_properties_len as usize);

    vkEnumerateInstanceExtensionProperties(
        std::ptr::null(), &mut instance_extension_properties_len, instance_extension_properties.as_mut_ptr()
    );
    
    return instance_extension_properties;
}}

impl crate::engine::Engine { pub fn create_instance(&mut self) { unsafe {
    for extension in EXTENSIONS() {
        get_instance_extension_properties().iter().position(|x| {
            let extension_name_u8: Vec<u8> = x.extension_name.iter().map(|y| *y as u8).filter(|y| *y != 0).collect();
            let extension_name_slice: &[u8] = &extension_name_u8;
            println!("{:?}", std::str::from_utf8(extension_name_slice).unwrap());
            return std::str::from_utf8(extension_name_slice).expect("Invalid extension name") == extension.as_c_str().to_str().unwrap()
        }).expect(format!("extension {:?} not supported!", extension).as_str());
    }

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
    
    let extensions = EXTENSIONS();
    
    let ptr1 = extensions[0].as_ptr();
    let ptr2 = extensions[1].as_ptr();

    let extension_ptrs: Vec<*const c_char> = vec![ptr1, ptr2];

    let extension_ptrs_ptr = extension_ptrs.as_ptr();

    let instance_create_info = VkInstanceCreateInfo {
        s_type: 1,
        p_next: std::ptr::null(),
        flags: 0,
        p_application_info: &application_info,
        enabled_layer_count: 0,
        pp_enabled_layer_names: std::ptr::null(),
        enabled_extension_count: extension_ptrs.len() as u32,
        pp_enabled_extension_names: extension_ptrs_ptr
    };

    let mut instance = 0;

    let result = vkCreateInstance(&instance_create_info, std::ptr::null(), &mut instance);

    if result == 0 {self.instance = instance; return;}

    panic!("InstanceCreation failed, error code: {}", result);
};}}
