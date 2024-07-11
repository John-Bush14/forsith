use std::error::Error;
use std::ffi::CString;
use ash::{vk, Entry, Instance, Device};
use ash::extensions::khr::{Surface, Win32Surface};
use ash::{
    extensions::ext::{DebugReport},
    version::{EntryV1_0, InstanceV1_0, DeviceV1_0},
};

pub mod validation;
mod device;
mod surface;

impl super::Engine {
    pub fn init(name: String, version: [u8;3], event_loop: fn(i8)) -> Result<Self, Box<dyn Error>> {
        let entry = ash::Entry::new().expect("Failed to create entry.");

        let app_info = vk::ApplicationInfo::builder()
            .application_name(CString::new(name)?.as_c_str())
            .application_version(ash::vk_make_version!(version[0], version[1], version[2]))
            .engine_name(CString::new("Forsith")?.as_c_str())
            .engine_version(ash::vk_make_version!(0, 1, 0))
            .api_version(ash::vk_make_version!(1, 0, 0))
            .build();

        let mut extension_names = vec![Surface::name().as_ptr(), Win32Surface::name().as_ptr()];
        if validation::ENABLE_VALIDATION_LAYERS {
            extension_names.push(DebugReport::name().as_ptr());
        }

        let (layer_names, layer_names_ptrs) = device::get_layer_names_and_pointers();        

        let mut instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);
        
        if validation::ENABLE_VALIDATION_LAYERS {
            validation::check_validation_layer_support(&entry);
            instance_create_info = instance_create_info.enabled_layer_names(&layer_names_ptrs);
        }

        let mut instance; unsafe { instance = entry.create_instance(&instance_create_info, None)?; }

        let debug_report_callback = validation::setup_debug_messenger(&entry, &instance);

        let physical_device = device::pick_physical_device(&instance);
        
        let (window, events_loop) = surface::create_window(name);

        let (surface, surface_khr) = unsafe { surface::create_surface(&window, entry, instance)? };

        let (device, graphics_queue) =
            device::create_logical_device_with_graphics_queue(&instance, physical_device);

        Ok(Self {
            entry: entry,
            instance: instance,
            debug_report_callback: debug_report_callback,
            physical_device: physical_device,
            device: device,
            graphics_queue: graphics_queue
        })
    }
}
