use ash::{vk, Instance, Device};
use ash::{
    extensions::ext::DebugReport,
    version::{EntryV1_0, InstanceV1_0, DeviceV1_0},
};

use std::ffi::{CStr, CString};

use super::validation;

pub fn pick_physical_device(instance: &Instance) -> vk::PhysicalDevice {
    let devices = unsafe { instance.enumerate_physical_devices().unwrap() };
    let device = devices
        .into_iter()
        .find(|device| is_device_suitable(instance, *device))
        .expect("No suitable physical device.");

    let props = unsafe { instance.get_physical_device_properties(device) };
    log::debug!("Selected physical device: {:?}", unsafe {
        CStr::from_ptr(props.device_name.as_ptr())
    });
    device
}

fn is_device_suitable(instance: &Instance, device: vk::PhysicalDevice) -> bool {
    find_queue_families(instance, device).is_some()
}

fn find_queue_families(instance: &Instance, device: vk::PhysicalDevice) -> Option<usize> {
    let props = unsafe { instance.get_physical_device_queue_family_properties(device) };
    props
        .iter()
        .enumerate()
        .find(|(_, family)| {
            family.queue_count > 0 && family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        })
        .map(|(index, _)| index)
}

pub fn get_layer_names_and_pointers() -> (Vec<CString>, Vec<*const i8>) {
    let layer_names = validation::REQUIRED_LAYERS
        .iter()
        .map(|name| CString::new(*name).expect("Failed to build CString"))
        .collect::<Vec<_>>();
    let layer_names_ptrs = layer_names
        .iter()
        .map(|name| name.as_ptr())
        .collect::<Vec<_>>();        

    (layer_names, layer_names_ptrs)
}

pub fn create_logical_device_with_graphics_queue(instance: &Instance, device: vk::PhysicalDevice) -> (Device, vk::Queue) {
    let queue_family_index = find_queue_families(instance, device).unwrap();
    let queue_priorities = [1.0f32];
    let queue_create_infos = [vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_index as u32)
        .queue_priorities(&queue_priorities)
        .build()];

    let device_features = vk::PhysicalDeviceFeatures::builder().build();

    let (_layer_names, layer_names_ptrs) = get_layer_names_and_pointers();

    let mut device_create_info_builder = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_features(&device_features);
    if validation::ENABLE_VALIDATION_LAYERS {
        device_create_info_builder =
            device_create_info_builder.enabled_layer_names(&layer_names_ptrs)
    }
    let device_create_info = device_create_info_builder.build();

    let device = unsafe {
        instance
            .create_device(device, &device_create_info, None)
            .expect("Failed to create logical device.")
    };
    let graphics_queue = unsafe { device.get_device_queue(queue_family_index as u32, 0) };

    (device, graphics_queue)
}
