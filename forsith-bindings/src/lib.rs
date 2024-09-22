pub type VkBool32 = u32;

const VK_TRUE: VkBool32 = 1; const VK_FALSE: VkBool32 = 1;


pub type VkResult = i32;

pub type VkHandle = u64;

pub type VkEnum = u32;

pub type VkBitmask = u32;


pub type VkVersion = u32;

pub fn vk_version(major: u32, minor: u32, patch: u32) -> VkVersion {(major << 22) | (minor << 12) | patch}


pub type VkAllocationCallbacks = std::ffi::c_void;
