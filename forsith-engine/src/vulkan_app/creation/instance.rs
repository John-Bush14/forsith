use bindings::{instance::{vkCreateInstance, VkApplicationInfo, VkInstance, VkInstanceCreateInfo}, structure_type::{VK_STRUCTURE_TYPE_APPLICATION_INFO, VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO}, VkVersion};
use std::ffi::CString;


pub(super) fn create_instance(app_name: &str, app_version: VkVersion) -> VkInstance {
    let app_name_c = CString::new(app_name).expect("invalid VulkanApp name!");

    let engine_name_c = CString::new(crate::ENGINE_NAME).expect("invalid engine name!");


    let application_info = VkApplicationInfo {
        s_type: VK_STRUCTURE_TYPE_APPLICATION_INFO,
        p_next: std::ptr::null(),
        p_application_name: app_name_c.as_ptr(),
        application_version: app_version,
        p_engine_name: engine_name_c.as_ptr(),
        engine_version: crate::ENGINE_VERSION(),
        api_version: crate::API_VERSION(),
    };


    let create_info = VkInstanceCreateInfo {
        s_type: VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: 0,
        p_application_info: &application_info as *const VkApplicationInfo,
        enabled_layer_count: 0,
        pp_enabled_layer_names: std::ptr::null(),
        enabled_extension_count: 0,
        pp_enabled_extension_names: std::ptr::null(),
    };


    let mut instance = 0;

    unsafe {vkCreateInstance(
        &create_info as *const VkInstanceCreateInfo,
        std::ptr::null(),
        &mut instance
    )};


    return instance;
}


#[cfg(test)]
mod instance_creation_tests {
    use super::create_instance;

    #[test]
    fn instance_creation_test() {
        let instance = create_instance("instance_creation_test", 0);

        assert!(instance != 0, "vkCreateInstance didn't modify instance, is still 0");
    }
}
