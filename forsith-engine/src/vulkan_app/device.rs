use bindings::{device::VkDevice, physical_device::{self, vk_enumerate_physical_devices, vk_get_physical_device_properties, vk_get_physical_device_queue_family_properties, VkPhysicalDevice, VkPhysicalDeviceProperties, VkPhysicalDeviceType, VkQueue, VkQueueFamily, VkQueueFamilyProperties, VkQueueFlagBits}};

use super::VulkanApp;


pub struct Queue {
    family: VkQueueFamily,
    queue: VkQueue,
    flags: VkQueueFlagBits
}

pub struct Device {
    physical_device: VkPhysicalDevice,
    queues: Vec<Queue>,
    device: VkDevice
}


fn rate_device_type(device_type: VkPhysicalDeviceType) -> u32 {
    match device_type {
        VkPhysicalDeviceType::VkPhysicalDeviceTypeOther => 2,
        VkPhysicalDeviceType::VkPhysicalDeviceTypeIntegratedGpu => 10,
        VkPhysicalDeviceType::VkPhysicalDeviceTypeDiscreteGpu => 6,
        VkPhysicalDeviceType::VkPhysicalDeviceTypeVirtualGpu => 8,
        VkPhysicalDeviceType::VkPhysicalDeviceTypeCpu => 4,
    }
}


impl VulkanApp {
    pub(crate) fn create_device(&self, queue_family_qualifiers: Vec<fn(VkPhysicalDevice, VkQueueFamilyProperties) -> bool>) -> Device {
        let physical_devices: Vec<VkPhysicalDevice> = vk_enumerate_physical_devices(self.instance);


        let (physical_device, queue_families) = physical_devices.into_iter()
            .map(|physical_device| -> Option<(VkPhysicalDevice, VkQueueFamily)> {
                let queue_families = vk_get_physical_device_queue_family_properties(physical_device);


                let mut qualifying_queue_families = Vec::with_capacity(queue_family_qualifiers.len());

                for queue_family_qualifier in &queue_family_qualifiers {
                    qualifying_queue_families.push(queue_families.iter().find(|&queue_familie| {
                        return queue_family_qualifier(physical_device, queue_familie.clone())
                    })?);
                }

                return None
            })
            .filter(|a| a.is_some()).map(|a| a.unwrap())
            .max_by_key(|(physical_device, _queue_families)| {
                let mut properties: VkPhysicalDeviceProperties = unsafe {std::mem::zeroed()};

                vk_get_physical_device_properties(*physical_device, &mut properties as *mut VkPhysicalDeviceProperties);

                return rate_device_type(properties.device_type)
            }).unwrap();


        todo!();
    }
}
