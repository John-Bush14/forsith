pub struct VkResult(i32);


impl From<&VkResult> for Result<(), VulkanError> {
    fn from(result: &VkResult) -> Result<(), VulkanError> {
        use VulkanError::*;

        if result.0 == 0 {return Ok(());}

        return Err(match result.0 {
            1 => NotReady,
            2 => Timeout,
            3 => EventSet,
            4 => EventReset,
            5 => Incomplete,
            -1 => OutOfHostMemory,
            -2 => OutOfDeviceMemory,
            -3 => InitializationFailed,
            -4 => DeviceLost,
            -5 => MemoryMapFailed,
            -6 => LayerNotPresent,
            -7 => ExtensionNotPresent,
            -8 => FeatureNotPresent,
            -9 => IncompatibleDriver,
            _ => UnknownError
        });
    }
}

impl VkResult {
    pub fn as_error(&self) -> Result<(), VulkanError> {return self.into();}
}


#[derive(Debug)]
pub enum VulkanError {
    NotReady,
    Timeout,
    EventSet,
    EventReset,
    Incomplete,
    OutOfHostMemory,
    OutOfDeviceMemory,
    InitializationFailed,
    DeviceLost,
    MemoryMapFailed,
    LayerNotPresent,
    ExtensionNotPresent,
    FeatureNotPresent,
    IncompatibleDriver,
    UnknownError
}


impl std::fmt::Display for VulkanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", match self {
            VulkanError::NotReady => "A fence or query has not yet completed.",
            VulkanError::Timeout => "A wait operation has not completed in the specified time.",
            VulkanError::EventSet => "An event is signaled.",
            VulkanError::EventReset => "An event is unsignaled.",
            VulkanError::Incomplete => "A return array was too small for the result.",
            VulkanError::OutOfHostMemory => "Host memory allocation has failed.",
            VulkanError::OutOfDeviceMemory => "Device memory allocation has failed.",
            VulkanError::InitializationFailed => "Initialization of an object could not be completed.",
            VulkanError::DeviceLost => "The logical or physical device has been lost.",
            VulkanError::MemoryMapFailed => "A memory map operation has failed.",
            VulkanError::LayerNotPresent => "A requested layer is not present.",
            VulkanError::ExtensionNotPresent => "A requested extension is not supported.",
            VulkanError::FeatureNotPresent => "A requested feature is not available.",
            VulkanError::IncompatibleDriver => "The requested version of Vulkan is not supported by the driver.",
            VulkanError::UnknownError => "An unknown error has occurred.",
        })
    }
}

impl std::error::Error for VulkanError {}
