pub mod instance;

pub mod structure_type;

pub mod macros;


pub type VkHandle = u64;

pub type VkEnum = u32;

pub type VkBitmask = u32;

pub type VkVersion = u32;

pub type VkResult = u32;

pub type VkBool32 = u32;


pub type VkAllocationCallbacks = *const std::ffi::c_void;
