use super::{CString, c_void, c_char};

pub const VK_CREATE_INSTANCE_CSTRING: fn() -> CString = || CString::new("vkCreateInstance").unwrap();

use super::VkResult;


pub type VkInstance = u64;

pub type VkDebugUtilsMessengerEXT = u64;


#[link(name = "vulkan")]
extern "C" { 
    pub fn vkCreateInstance(
        instance_create_info: *const VkInstanceCreateInfo, _: *const c_void, instance: *mut VkInstance
    ) -> VkResult;

    pub fn vkDestroyInstance(
        instance: VkInstance, _: *const c_void
    ) -> VkResult;

    pub fn vkEnumerateInstanceExtensionProperties(
        layer_name: *const c_char,
        extension_property_count: *mut u32,
        extension_properties: *mut VkExtensionProperties
    ) -> VkResult;

    pub fn vkEnumerateInstanceLayerProperties(
        property_count: *mut u32,
        properties: *mut VkLayerProperties
    ) -> VkResult;

    pub fn vkCreateDebugUtilsMessengerEXT(
        instance: VkInstance,
        create_info: *const VkDebugUtilsMessengerCreateInfoEXT,
        _: *const c_void,
        messenger: *mut VkDebugUtilsMessengerEXT
    ) -> VkResult;
}


#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct VkLayerProperties {
    pub extension_name: [c_char; 256], // so macro works
    pub spec_version: u32,
    pub implementation_version: u32,
    pub description: [c_char; 256]
}


#[repr(C)]
pub struct VkDebugUtilsMessengerCreateInfoEXT {
    pub s_type: super::VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub severity: u32,
    pub type_: u32,
    pub fn_callback: unsafe extern "system" fn(u32, u32, *const VkDebugUtilsMessengerCallbackDataEXT, *mut c_void) -> super::VkBool32,
    pub user_data: *const c_void
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct VkDebugUtilsMessengerCallbackDataEXT {
    pub s_type: super::VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub message_id_name: *const c_char,
    pub message_id_number: i32,
    pub message: *const c_char,
    pub queue_label_count: u32,
    pub queue_labels: *const VkDebugUtilsLabelEXT,
    pub cmd_buf_label_count: u32,
    pub cmd_buf_labels: *const VkDebugUtilsLabelEXT,
    pub object_count: u32,
    pub objects: *const VkDebugUtilsObjectNameInfoEXT
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct VkDebugUtilsLabelEXT {
    pub s_type: super::VkStructureType,
    pub p_next: *const c_void,
    pub label_name: *const c_char,
    pub color: [f32;4]
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct VkDebugUtilsObjectNameInfoEXT {
    pub s_type: super::VkStructureType,
    pub p_next: *const c_void,
    pub object_type: u32,
    pub object_handle: u64,
    pub object_name: *const c_char
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct VkExtensionProperties {
    pub extension_name: [c_char; 256],
    pub spec_version: u32
}

#[repr(C)]
pub struct VkApplicationInfo {
    pub s_type: super::VkStructureType,
    pub p_next: *const c_void,
    pub p_application_name: *const c_char,
    pub application_version: u32,
    pub p_engine_name: *const c_char,
    pub engine_version: u32,
    pub api_version: u32,
}

#[repr(C)]
pub struct VkInstanceCreateInfo {
    pub s_type: super::VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub p_application_info: *const VkApplicationInfo,
    pub enabled_layer_count: u32,
    pub pp_enabled_layer_names: *const *const c_char,
    pub enabled_extension_count: u32,
    pub pp_enabled_extension_names: *const *const c_char,
}
