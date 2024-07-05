pub use std::ffi::CString;
use std::ptr;
pub use std::os::raw::{c_void, c_char};
use std::sync::{Once, ONCE_INIT, Mutex};

pub mod abstractions;

pub mod initialisation;
pub use initialisation::{VkInstance, CreateInstance, CreateDevice, VkDevice, CreateWindow};

pub type VkResult = i32;

pub type VkStructureType = u32;

extern "C" {
    fn dlopen(filename: *const c_char, flag: i32) -> *mut c_void;
    pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    pub fn dlclose(handle: *mut c_void) -> i32;
}

pub fn vk_make_version(major: u32, minor: u32, patch: u32) -> u32 {
    ((major << 22) | (minor << 12) | patch)
}

pub unsafe fn load_vulkan_function(name: CString) -> *const c_void {
    let func_ptr = dlsym(get_lib(), name.as_ptr());
    if func_ptr.is_null() {
        panic!("Failed to load function: {:?}", name);
    }
    return func_ptr;
}

const RTLD_NOW: i32 = 2;

static mut VULKAN_LIB: Option<*mut c_void> = None;
static mut INSTANCE: Option<VkInstance> = None;
static mut DEVICE: Option<VkDevice> = None;
static LIB_ONCE: Once = ONCE_INIT;
static INST_ONCE: Once = ONCE_INIT;
static DEVICE_ONCE: Once = ONCE_INIT;

pub fn get_lib() -> *mut c_void {
    unsafe {
        LIB_ONCE.call_once(|| {
            let lib_source = CString::new("libvulkan.so.1").unwrap();

            VULKAN_LIB = Some(dlopen(lib_source.as_ptr(), RTLD_NOW));
            if VULKAN_LIB.unwrap().is_null() {panic!("failed to load vulkan lib!");}
        });
        // Return the stored result
        return VULKAN_LIB.unwrap()
    }
}

pub fn get_instance() -> VkInstance {unsafe {
    INST_ONCE.call_once(|| {
        INSTANCE = Some(CreateInstance("Test".to_string(), 0))
    });

    return INSTANCE.unwrap()
}}

pub fn get_device() -> VkDevice {unsafe {
    DEVICE_ONCE.call_once(|| {
        DEVICE = Some(CreateDevice())
    });

    return DEVICE.unwrap()
}}
