use std::fmt;


impl From<VkResult> for Result<VkResult, VkResult> {
    fn from(result: VkResult) -> Result<VkResult, VkResult> {
        if result.clone() as i32 >= 0 {return Ok(result);}
        return Err(result);
    }
}

impl VkResult {
    pub fn result(self) -> Result<VkResult, VkResult> {self.into()}
}


#[repr(i32)]
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum VkResult {
    VkSuccess = 0,
    VkNotReady = 1,
    VkTimeout = 2,
    VkEventSet = 3,
    VkEventReset = 4,
    VkIncomplete = 5,
    VkErrorOutOfHostMemory = -1,
    VkErrorOutOfDeviceMemory = -2,
    VkErrorInitializationFailed = -3,
    VkErrorDeviceLost = -4,
    VkErrorMemoryMapFailed = -5,
    VkErrorLayerNotPresent = -6,
    VkErrorExtensionNotPresent = -7,
    VkErrorFeatureNotPresent = -8,
    VkErrorIncompatibleDriver = -9,
    VkErrorTooManyObjects = -10,
    VkErrorFormatNotSupported = -11,
    VkErrorFragmentedPool = -12,
    VkErrorUnknown = -13,

    // Khronos and EXT extension errors
    VkErrorOutOfPoolMemory = -1000069000,
    VkErrorInvalidExternalHandle = -1000072003,
    VkErrorFragmentation = -1000161000,
    VkErrorInvalidOpaqueCaptureAddress = -1000257000,

    // VK_KHR_surface
    VkErrorSurfaceLostKhr = -1000000000,
    VkErrorNativeWindowInUseKhr = -1000000001,

    // VK_KHR_swapchain
    VkSuboptimalKhr = 1000001003,
    VkErrorOutOfDateKhr = -1000001004,

    // VK_KHR_display_swapchain
    VkErrorIncompatibleDisplayKhr = -1000003001,

    // VK_EXT_debug_report
    VkErrorValidationFailedExt = -1000011001,

    // VK_NV_glsl_shader
    VkErrorInvalidShaderNv = -1000012000,

    // VK_EXT_image_drm_format_modifier
    VkErrorInvalidDrmFormatModifierPlaneLayoutExt = -1000158000,

    // VK_EXT_global_priority
    VkErrorNotPermittedExt = -1000174001,

    // VK_KHR_ray_tracing_pipeline
    VkErrorPipelineCompileRequiredExt = 1000297000,

    // VK_KHR_deferred_host_operations
    VkThreadIdleKhr = 1000268000,
    VkThreadDoneKhr = 1000268001,
    VkOperationDeferredKhr = 1000268002,
    VkOperationNotDeferredKhr = 1000268003,

    // VK_EXT_full_screen_exclusive
    VkErrorFullScreenExclusiveModeLostExt = -1000255000,

    // VK_EXT_image_compression_control
    VkErrorImageCompressionNotAcceptableExt = -1000338000,

    // VK_EXT_image_view_min_lod
    VkErrorImageViewMinLodInvalidExt = -1000414000,

    // VK_EXT_maintenance4
    VkErrorFragmentDensityMapResetExt = -1000349000,
}

impl fmt::Display for VkResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match *self {
            VkResult::VkSuccess => "Success",
            VkResult::VkNotReady => "Not Ready",
            VkResult::VkTimeout => "Timeout",
            VkResult::VkEventSet => "Event Set",
            VkResult::VkEventReset => "Event Reset",
            VkResult::VkIncomplete => "Incomplete",
            VkResult::VkErrorOutOfHostMemory => "Out of Host Memory",
            VkResult::VkErrorOutOfDeviceMemory => "Out of Device Memory",
            VkResult::VkErrorInitializationFailed => "Initialization Failed",
            VkResult::VkErrorDeviceLost => "Device Lost",
            VkResult::VkErrorMemoryMapFailed => "Memory Map Failed",
            VkResult::VkErrorLayerNotPresent => "Layer Not Present",
            VkResult::VkErrorExtensionNotPresent => "Extension Not Present",
            VkResult::VkErrorFeatureNotPresent => "Feature Not Present",
            VkResult::VkErrorIncompatibleDriver => "Incompatible Driver",
            VkResult::VkErrorTooManyObjects => "Too Many Objects",
            VkResult::VkErrorFormatNotSupported => "Format Not Supported",
            VkResult::VkErrorFragmentedPool => "Fragmented Pool",
            VkResult::VkErrorUnknown => "Unknown Error",
            VkResult::VkErrorOutOfPoolMemory => "Out of Pool Memory",
            VkResult::VkErrorInvalidExternalHandle => "Invalid External Handle",
            VkResult::VkErrorFragmentation => "Fragmentation",
            VkResult::VkErrorInvalidOpaqueCaptureAddress => "Invalid Opaque Capture Address",
            VkResult::VkErrorSurfaceLostKhr => "Surface Lost",
            VkResult::VkErrorNativeWindowInUseKhr => "Native Window In Use",
            VkResult::VkSuboptimalKhr => "Suboptimal",
            VkResult::VkErrorOutOfDateKhr => "Out of Date",
            VkResult::VkErrorIncompatibleDisplayKhr => "Incompatible Display",
            VkResult::VkErrorValidationFailedExt => "Validation Failed",
            VkResult::VkErrorInvalidShaderNv => "Invalid Shader",
            VkResult::VkErrorInvalidDrmFormatModifierPlaneLayoutExt => "Invalid DRM Format Modifier Plane Layout",
            VkResult::VkErrorNotPermittedExt => "Not Permitted",
            VkResult::VkErrorPipelineCompileRequiredExt => "Pipeline Compile Required",
            VkResult::VkThreadIdleKhr => "Thread Idle",
            VkResult::VkThreadDoneKhr => "Thread Done",
            VkResult::VkOperationDeferredKhr => "Operation Deferred",
            VkResult::VkOperationNotDeferredKhr => "Operation Not Deferred",
            VkResult::VkErrorFullScreenExclusiveModeLostExt => "Full Screen Exclusive Mode Lost",
            VkResult::VkErrorImageCompressionNotAcceptableExt => "Image Compression Not Acceptable",
            VkResult::VkErrorImageViewMinLodInvalidExt => "Image View Min LOD Invalid",
            VkResult::VkErrorFragmentDensityMapResetExt => "Fragment Density Map Reset",
        };

        write!(f, "{}", message)
    }
}

// Implementing std::error::Error for VkResult
impl std::error::Error for VkResult {}
