use ash::{
    extensions::ext::DebugReport,
    version::{EntryV1_0, InstanceV1_0},
};
use ash::{vk, Entry, Instance};
use core::ffi::{CStr, c_void, c_char};

#[cfg(debug_assertions)]
pub const ENABLE_VALIDATION_LAYERS: bool = true;

#[cfg(not(debug_assertions))]
pub const ENABLE_VALIDATION_LAYERS: bool = false;

pub const REQUIRED_LAYERS: [&'static str; 1] = ["VK_LAYER_LUNARG_standard_validation"];

pub unsafe extern "system" fn vulkan_debug_callback(
    flag: vk::DebugReportFlagsEXT,
    typ: vk::DebugReportObjectTypeEXT,
    _: u64,
    _: usize,
    _: i32,
    _: *const c_char,
    p_message: *const c_char,
    _: *mut c_void,
) 

-> u32 {
    match flag {
        vk::DebugReportFlagsEXT::DEBUG => {log::debug!("{} - {:?}", typ, CStr::from_ptr(p_message));},
        vk::DebugReportFlagsEXT::INFORMATION => {log::info!("{} - {:?}", typ, CStr::from_ptr(p_message));},
        vk::DebugReportFlagsEXT::WARNING => {log::warn!("{} - {:?}", typ, CStr::from_ptr(p_message));},
        vk::DebugReportFlagsEXT::PERFORMANCE_WARNING => {log::warn!("{} - {:?}", typ, CStr::from_ptr(p_message));},
        _ => {log::error!("{} - {:?}", typ, CStr::from_ptr(p_message));}
    } return vk::FALSE
}

pub fn check_validation_layer_support(entry: &Entry) {
    for required in REQUIRED_LAYERS.iter() {
        let found = entry
            .enumerate_instance_layer_properties()
            .unwrap()
            .iter()
            .any(|layer| {
                let name = unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) };
                let name = name.to_str().expect("Failed to get layer name pointer");
                required == &name
            });

        if !found {
            panic!("Validation layer not supported: {}", required);
        }
    }
}

pub fn setup_debug_messenger(entry: &Entry, instance: &Instance) -> Option<(DebugReport, vk::DebugReportCallbackEXT)> {
    if !ENABLE_VALIDATION_LAYERS {
        return None;
    }
    let create_info = vk::DebugReportCallbackCreateInfoEXT::builder()
        .flags(vk::DebugReportFlagsEXT::all())
        .pfn_callback(Some(vulkan_debug_callback))
        .build();
    let debug_report = DebugReport::new(entry, instance);
    let debug_report_callback = unsafe {
        debug_report
            .create_debug_report_callback(&create_info, None)
            .unwrap()
    };
    Some((debug_report, debug_report_callback))
}
