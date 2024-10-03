use std::collections::HashMap;

use bindings::{device::{vk_create_device, vk_destroy_device, vk_get_device_queue, VkDevice, VkDeviceCreateInfo, VkDeviceQueueCreateInfo}, instance::VkInstance, physical_device::{vk_enumerate_physical_devices, vk_get_physical_device_properties, vk_get_physical_device_queue_family_properties, VkPhysicalDevice, VkPhysicalDeviceProperties, VkPhysicalDeviceType, VkQueue, VkQueueFamily, VkQueueFamilyProperties}, vk_result::VkResult};

use crate::DynError;


#[allow(dead_code)]
pub struct Queue {
    family: VkQueueFamily,
    queue: VkQueue,
}

#[allow(dead_code)]
pub struct Device {
    physical_device: VkPhysicalDevice,
    queues: Vec<Queue>,
    device: VkDevice
}


impl Device {pub(crate) fn destroy(&self) -> Result<(), DynError> {
    if self.device == 0 {return Err(Box::new(VkResult::VkErrorDeviceLost));}

    vk_destroy_device(self.device, std::ptr::null());

    return Ok(());
}}


fn rate_device_type(device_type: VkPhysicalDeviceType) -> u32 {
    match device_type {
        VkPhysicalDeviceType::VkPhysicalDeviceTypeOther => 2,
        VkPhysicalDeviceType::VkPhysicalDeviceTypeIntegratedGpu => 10,
        VkPhysicalDeviceType::VkPhysicalDeviceTypeDiscreteGpu => 6,
        VkPhysicalDeviceType::VkPhysicalDeviceTypeVirtualGpu => 8,
        VkPhysicalDeviceType::VkPhysicalDeviceTypeCpu => 4,
    }
}


pub(crate) fn create_device(
    instance: VkInstance,
    queue_family_qualifiers: Vec<fn(VkPhysicalDevice, VkQueueFamilyProperties) -> bool>
) -> Result<Device, DynError> {
    let physical_devices: Vec<VkPhysicalDevice> = vk_enumerate_physical_devices(instance);


    let (physical_device, queue_families) = physical_devices.into_iter()
        .map(|physical_device| -> Option<(VkPhysicalDevice, Vec<u32>)> {
            let queue_families = vk_get_physical_device_queue_family_properties(physical_device);


            let mut qualifying_queue_families = Vec::with_capacity(queue_family_qualifiers.len());

            for queue_family_qualifier in &queue_family_qualifiers {
                let (qualifying_queue_family, _) = queue_families.iter().enumerate().find(|(_i, &ref queue_familie)| {
                    return queue_family_qualifier(physical_device, queue_familie.clone())
                })?;

                qualifying_queue_families.push(qualifying_queue_family as u32);
            }

            return Some((physical_device, qualifying_queue_families));
        })
        .filter(|a| a.is_some()).map(|a| a.unwrap())
        .max_by_key(|(physical_device, _queue_families)| {
            let mut properties: VkPhysicalDeviceProperties = unsafe {std::mem::zeroed()};

            vk_get_physical_device_properties(*physical_device, &mut properties as *mut VkPhysicalDeviceProperties);

            return rate_device_type(properties.device_type)
        }).unwrap();


    let priority = 1.0f32;

    let queue_create_infos = queue_families.iter().map(|queue_family| {
        return VkDeviceQueueCreateInfo {
            s_type: VkDeviceQueueCreateInfo::structure_type(),
            p_next: std::ptr::null(),
            flags: bindings::device::VkDeviceQueueCreateFlags(0),
            queue_family_index: *queue_family,
            queue_count: 1,
            queue_priorities: &priority,
        }}
    ).collect::<Vec<VkDeviceQueueCreateInfo>>();


    let mut vk_device = 0;

    let create_info = VkDeviceCreateInfo {
        s_type: VkDeviceCreateInfo::structure_type(),
        p_next: std::ptr::null(),
        flags: bindings::device::VkDeviceCreateFlags(0),
        queue_create_info_count: queue_families.len() as u32,
        queue_create_infos: queue_create_infos.as_ptr(),
        enabled_layer_count: 0,
        enabled_layer_names: std::ptr::null(),
        enabled_extension_count: 0,
        enabled_extensions: std::ptr::null(),
        enabled_features: std::ptr::null(),
    };

    vk_create_device(physical_device, &create_info as *const VkDeviceCreateInfo, std::ptr::null(), &mut vk_device).result()?;


    let mut queue_count: HashMap<VkQueueFamily, u32> = HashMap::new();

    let queues = queue_families.iter().map(|queue_family| {
        let mut count = 0;

        if let Some(actual_count) = queue_count.get_mut(queue_family) {
            *actual_count += 1;
            count = *actual_count;
        } else {queue_count.insert(*queue_family, 0);}

        let mut queue = 0;

        vk_get_device_queue(vk_device, *queue_family, count, &mut queue);


        return Queue {
            family: *queue_family,
            queue,
        };
    }).collect::<Vec<Queue>>();


    return Ok(Device {
        physical_device,
        queues,
        device: vk_device,
    });
}


#[cfg(test)]
mod device_tests {
    use bindings::{instance::vk_destroy_instance, vk_version};

    use crate::{vulkan_app::creation::instance::create_instance, DynError};

    use super::create_device;

    #[test]
    fn test_device_creation_and_destroyal() -> Result<(), DynError> {
        let instance = create_instance("device creation test", vk_version(0, 0, 0)).expect("device creation did not fail, instance creation did");

        create_device(instance, vec![|_, _| return true])?.destroy()?;

        vk_destroy_instance(instance, std::ptr::null());

        return Ok(());
    }
}
