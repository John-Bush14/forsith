use bindings::{instance::{vk_create_instance, VkApplicationInfo, VkInstance, VkInstanceCreateFlags, VkInstanceCreateInfo, vk_enumerate_instance_extension_properties}, VkVersion};
use std::ffi::{c_char, CString};
use crate::{errors::ForsithError, DynError, ENGINE_NAME, ENGINE_VERSION};

use crate::API_VERSION;


pub(crate) fn create_instance(app_name: &str, app_version: VkVersion) -> Result<VkInstance, DynError> {
    let c_app_name = CString::new(app_name)?;

    let engine_name = ENGINE_NAME();


    let app_info = VkApplicationInfo {
        s_type: VkApplicationInfo::structure_type(),
        p_next: std::ptr::null(),
        application_name: c_app_name.as_ptr(),
        application_version: app_version,
        engine_name: engine_name.as_ptr(),
        engine_version: ENGINE_VERSION(),
        api_version: API_VERSION(),
    };


    let supported_extensions = vk_enumerate_instance_extension_properties(std::ptr::null()).iter().map(|props| {
        CString::new(props.extension_name.iter().filter(|char| **char != 0).map(|c| *c as u8).collect::<Vec<u8>>()).unwrap()
    }).collect::<Vec<CString>>();

    let extensions = [
        CString::new("VK_KHR_surface")?,
    ];

    for ext in extensions.iter() {
        if !supported_extensions.contains(ext) {
            return Err(Box::new(ForsithError::InstanceExtensionNotPresent(ext.clone())));
        }
    }

    let extension_ptrs = extensions.iter().map(|ext| return ext.as_ptr()).collect::<Vec<*const c_char>>();


    let instance_info = VkInstanceCreateInfo {
        s_type: VkInstanceCreateInfo::structure_type(),
        p_next: std::ptr::null(),
        flags: VkInstanceCreateFlags(0),
        application_info: &app_info as *const VkApplicationInfo,
        enabled_layer_count: 0,
        enabled_layer_names: std::ptr::null(),
        enabled_extension_count: extensions.len() as u32,
        enabled_extensions: extension_ptrs.as_ptr()
    };


    let mut instance = 0;

    vk_create_instance(&instance_info as *const VkInstanceCreateInfo, std::ptr::null(), &mut instance).result()?;

    assert!(instance != 0);


    return Ok(instance);
}
