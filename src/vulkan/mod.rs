use std::ffi::CString;
use std::ptr;
use std::os::raw::{c_void, c_char};
use std::sync::{Once, ONCE_INIT, Mutex};

pub mod abstractions;

pub mod initialisation;
pub use initialisation::{VkInstance, CreateInstance};

pub type VkResult = i32;

extern "C" {
    fn dlopen(filename: *const c_char, flag: i32) -> *mut c_void;
    pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    pub fn dlclose(handle: *mut c_void) -> i32;
}

pub fn vk_make_version(major: u32, minor: u32, patch: u32) -> u32 {
    ((major << 22) | (minor << 12) | patch)
}

const RTLD_NOW: i32 = 2;

static mut VULKAN_LIB: Option<*mut c_void> = None;
static mut INSTANCE: Option<VkInstance> = None;
static LIB_ONCE: Once = ONCE_INIT;
static INST_ONCE: Once = ONCE_INIT;

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
