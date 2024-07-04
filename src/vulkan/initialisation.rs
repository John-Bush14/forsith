pub type VkInstance = u64;
use super::{CString, c_void, c_char, vk_make_version};

#[repr(C)]
pub struct VkApplicationInfo {
    s_type: u32,
    p_next: *const c_void,
    p_application_name: *const c_char,
    application_version: u32,
    p_engine_name: *const c_char,
    engine_version: u32,
    api_version: u32,
}

#[repr(C)]
pub struct VkInstanceCreateInfo {
    s_type: u32,
    p_next: *const c_void,
    flags: u32,
    p_application_info: *const VkApplicationInfo,
    enabled_layer_count: u32,
    pp_enabled_layer_names: *const *const c_char,
    enabled_extension_count: u32,
    pp_enabled_extension_names: *const *const c_char,
}

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
        s_type: 0,
        p_next: std::ptr::null(),
        flags: 0,
        p_application_info: &application_info,
        enabled_layer_count: 0,
        pp_enabled_layer_names: std::ptr::null(),
        enabled_extension_count: 0,
        pp_enabled_extension_names: std::ptr::null()
    };

    let mut instance = 0;
    panic!("before");

    let vulkan_lib = super::get_lib();
    
    let vk_create_instance_cstring = CString::new("VkCreateInstance").unwrap();
    
    panic!("");

    let vk_create_instance: extern "C" fn(
        *const VkInstanceCreateInfo, *const c_void, *mut VkInstance
    ) -> super::VkResult 

     = std::mem::transmute(super::dlsym(
        vulkan_lib,
        vk_create_instance_cstring.as_ptr()
    ));

    panic!("{:?}", vk_create_instance);

    let result = vk_create_instance(&instance_create_info, std::ptr::null(), &mut instance);

    if result != 0 {return instance}

    panic!("InstanceCreation failed");
};}
