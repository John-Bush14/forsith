use bindings::{instance::{vk_create_instance, VkApplicationInfo, VkInstance, VkInstanceCreateFlags, VkInstanceCreateInfo}, vk_version, VkVersion};
use std::ffi::CString;
use crate::DynError;

use super::API_VERSION;


pub fn create_instance(app_name: &str, app_version: VkVersion) -> Result<VkInstance, DynError> {
    let c_app_name = CString::new(app_name)?;

    let engine_name = CString::new("forsith")?;


    let app_info = VkApplicationInfo {
        s_type: VkApplicationInfo::structure_type(),
        p_next: std::ptr::null(),
        application_name: c_app_name.as_ptr(),
        application_version: app_version,
        engine_name: engine_name.as_ptr(),
        engine_version: vk_version(0, 1, 0),
        api_version: API_VERSION(),
    };


    let instance_info = VkInstanceCreateInfo {
        s_type: VkInstanceCreateInfo::structure_type(),
        p_next: std::ptr::null(),
        flags: VkInstanceCreateFlags(0),
        application_info: &app_info as *const VkApplicationInfo,
        enabled_layer_count: 0,
        enabled_layer_names: std::ptr::null(),
        enabled_extension_count: 0,
        enabled_extensions: std::ptr::null()
    };


    let mut instance = 0;

    vk_create_instance(&instance_info as *const VkInstanceCreateInfo, std::ptr::null(), &mut instance).result()?;


    return Ok(instance);
}
