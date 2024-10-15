use bindings::Bitmask;

use bindings::{device::{vk_create_device, vk_destroy_device, vk_get_device_queue, VkDevice, VkDeviceCreateInfo, VkDeviceQueueCreateInfo}, instance::VkInstance, physical_device::{vk_enumerate_physical_devices, vk_get_physical_device_properties, vk_get_physical_device_queue_family_properties, VkPhysicalDevice, VkPhysicalDeviceProperties, VkPhysicalDeviceType, VkQueue, VkQueueFamily, VkQueueFlagBits}, vk_result::VkResult};

use crate::DynError;

use super::creation::VulkanAppLimits;


#[allow(dead_code)]
pub struct Queue {
    family: VkQueueFamily,
    queue: VkQueue,
}

#[allow(dead_code)]
pub struct RenderQueueSet {
    presentation: Queue,
    graphics: Queue
}

#[allow(dead_code)]
pub struct Device {
    physical_device: VkPhysicalDevice,
    render_queue_sets: Vec<RenderQueueSet>,
    general_graphics_queue: Queue,
    device: VkDevice
}


impl Device {pub(crate) fn destroy(&self) -> Result<(), DynError> {
    if self.device == 0 {return Err(Box::new(VkResult::VkErrorDeviceLost));}

    vk_destroy_device(self.device, std::ptr::null());

    return Ok(());
}}

#[allow(dead_code)]
impl Device {pub(crate) fn get_render_queue_set(&self, i: usize) -> &RenderQueueSet {return &self.render_queue_sets[i];}}
#[allow(dead_code)]
impl Device {pub(crate) fn get_device(&self) -> &VkDevice {return &self.device;}}
#[allow(dead_code)]
impl Queue {pub(crate) fn family(&self) -> VkQueueFamily {return self.family;}}


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
    app_limits: &VulkanAppLimits
) -> Result<Device, DynError> {
    let needed_render_sets = app_limits.get_renderers() as usize;


    let physical_devices: Vec<VkPhysicalDevice> = vk_enumerate_physical_devices(instance);


    let (physical_device, queue_families) = physical_devices.into_iter()
        .filter_map(|physical_device| -> Option<(VkPhysicalDevice, (Vec<(VkQueueFamily, usize)>, Vec<(VkQueueFamily, usize)>))> {
            let queue_families = vk_get_physical_device_queue_family_properties(physical_device);

            let mut graphics_queue_families: Vec<(VkQueueFamily, usize)> = vec!();

            let mut presentation_queue_families: Vec<(VkQueueFamily, usize)> = vec!();

            for (queue_family, queue_family_props) in queue_families.iter().enumerate() {
                let mut queue_count = queue_family_props.queue_count;

                if queue_family_props.queue_flags.contains(VkQueueFlagBits::VkQueueGraphicsBit) {
                    let graphics_queue_count = (needed_render_sets - graphics_queue_families.len()).min(0).max(queue_count as usize);
                    queue_count -= graphics_queue_count as u32;

                    graphics_queue_families.push((queue_family as VkQueueFamily, graphics_queue_count));
                };
            }

            if graphics_queue_families.len() < needed_render_sets {return None;}
            if presentation_queue_families.len() < needed_render_sets {return None;}

            return Some((physical_device, (graphics_queue_families, presentation_queue_families)));
        })
        .max_by_key(|(physical_device, _)| {
            let mut properties: VkPhysicalDeviceProperties = unsafe {std::mem::zeroed()};

            vk_get_physical_device_properties(*physical_device, &mut properties as *mut VkPhysicalDeviceProperties);

            return rate_device_type(properties.device_type)
        }).unwrap();


    let priority = [1.0f32; 256];

    let queue_create_infos = queue_families.0.iter().zip(queue_families.1.iter()).map(|queue_family| {
        return (
            VkDeviceQueueCreateInfo {
                s_type: VkDeviceQueueCreateInfo::structure_type(),
                p_next: std::ptr::null(),
                flags: bindings::device::VkDeviceQueueCreateFlags(0),
                queue_family_index: queue_family.0.0,
                queue_count: queue_family.1.1 as u32,
                queue_priorities: priority.as_ptr(),
            },
            VkDeviceQueueCreateInfo {
                s_type: VkDeviceQueueCreateInfo::structure_type(),
                p_next: std::ptr::null(),
                flags: bindings::device::VkDeviceQueueCreateFlags(0),
                queue_family_index: queue_family.1.0,
                queue_count: queue_family.1.1 as u32,
                queue_priorities: priority.as_ptr(),
            },
        );}
    ).collect::<Vec<(VkDeviceQueueCreateInfo, VkDeviceQueueCreateInfo)>>();


    let mut vk_device = 0;

    let create_info = VkDeviceCreateInfo {
        s_type: VkDeviceCreateInfo::structure_type(),
        p_next: std::ptr::null(),
        flags: bindings::device::VkDeviceCreateFlags(0),
        queue_create_info_count: queue_create_infos.len() as u32,
        queue_create_infos: queue_create_infos.as_ptr() as *const VkDeviceQueueCreateInfo,
        enabled_layer_count: 0,
        enabled_layer_names: std::ptr::null(),
        enabled_extension_count: 0,
        enabled_extensions: std::ptr::null(),
        enabled_features: std::ptr::null(),
    };


    vk_create_device(physical_device, &create_info as *const VkDeviceCreateInfo, std::ptr::null(), &mut vk_device).result()?;

    assert!(vk_device != 0);


    let mut render_queue_sets = vec!();

    let mut graphics_cord = (0, 0); let mut present_cord = (0, 0);

    for _ in 0..needed_render_sets {
        if queue_families.0.len() >= graphics_cord.1 {graphics_cord.1 = 0; graphics_cord.0 += 1;} else {graphics_cord.1 += 1;}
        if queue_families.1.len() >= present_cord.1 {present_cord.1 = 0; present_cord.0 += 1;} else {present_cord.1 += 1;}


        let mut queue = 0;

        vk_get_device_queue(vk_device, queue_families.0[graphics_cord.0].0, graphics_cord.1 as u32, &mut queue);

        assert!(queue != 0);

        let graphics_queue = queue;

        vk_get_device_queue(vk_device, queue_families.1[present_cord.0].0, present_cord.1 as u32, &mut queue);

        assert!(queue != 0);

        let present_queue = queue;


        render_queue_sets.push(
            RenderQueueSet {
                presentation: Queue {
                    family: queue_families.0[graphics_cord.0].0,
                    queue: graphics_queue,
                },
                graphics: Queue {
                    family: queue_families.1[present_cord.0].0,
                    queue: present_queue,
                }
            }
        );
    }


    return Ok(Device {
        physical_device,
        render_queue_sets,
        general_graphics_queue: todo!(),
        device: vk_device,
    });
}


#[cfg(test)]
mod device_tests {
    use bindings::{instance::vk_destroy_instance, vk_version};

    use crate::{vulkan_app::creation::{instance::create_instance, VulkanAppLimits}, DynError};

    use super::create_device;

    #[test]
    fn test_device_creation_and_destroyal() -> Result<(), DynError> {
        let instance = create_instance("device creation test", vk_version(0, 0, 0)).expect("device creation did not fail, instance creation did");

        create_device(instance, &VulkanAppLimits::default())?.destroy()?;

        vk_destroy_instance(instance, std::ptr::null());

        return Ok(());
    }
}
