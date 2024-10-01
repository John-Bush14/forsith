pub mod vk_result;

pub mod macros;

pub mod structure_type;


pub mod instance;

pub mod device;

pub mod physical_device;

pub mod command_pool;


/// vulkan 32 bit bool type
pub type VkBool32 = u32;

/// 32 bit true (1)
pub const VK_TRUE: VkBool32 = 1;
/// 32 bit false (0)
pub const VK_FALSE: VkBool32 = 1;

/// type of vulkan handles
pub type VkHandle = u64;

/// type of vulkan enums
pub type VkEnum = u32;

/// type of vulkan bitmasks
pub type VkBitmask = u32;
/// type of vulkan bitflags (equal to vulkan bitmasks)
pub type VkBitflag = VkBitmask;

/// trait implemented by bitmasks
pub trait Bitmask {
    type Bitflag;

    fn contains(&self, bitflag: Self::Bitflag) -> bool;
}

/// type of vulkan versions
pub type VkVersion = u32;
/// create vulkan version from major, minor and patch
pub fn vk_version(major: u32, minor: u32, patch: u32) -> VkVersion {(major << 22) | (minor << 12) | patch}

/// information about memory management
pub type VkAllocationCallbacks = std::ffi::c_void;


define_vk_structs!(
    pub VkExtent3D {
        width: u32,
        height: u32,
        depth: u32
    }
);
