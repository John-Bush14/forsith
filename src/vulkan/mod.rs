pub use std::ffi::CString;
use std::ptr;
pub use std::os::raw::{c_void, c_char};


pub type VkResult = i32;

pub type VkStructureType = u32;

pub type VkBool32 = u32;

const RTLD_NOW: i32 = 2;


extern "C" {
    fn dlopen(filename: *const c_char, flag: i32) -> *mut c_void;
    pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    pub fn dlclose(handle: *mut c_void) -> i32;
}

pub fn vk_make_version(major: u32, minor: u32, patch: u32) -> u32 {
    ((major << 22) | (minor << 12) | patch)
}


pub mod rendering;

pub mod macros;

pub mod pipeline;

pub mod devices;

pub mod window;

pub mod uniform;

pub mod instance;

pub mod swapchain;

pub mod vertex;

pub mod commands;
