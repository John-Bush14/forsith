use super::{c_void, c_char};

use crate::vulkan::{
    instance::VkInstance,
    window::VkSurfaceKHR,
    VkResult,
    VkBool32
};


pub type VkPhysicalDevice = u64;

pub type VkPhysicalDeviceType = u32;


#[link(name = "vulkan")]
extern "C" {
    pub fn vkEnumeratePhysicalDevices(
        instance: VkInstance,
        physical_device_count: *mut u32,
        physical_devices: *mut VkPhysicalDevice
    ) -> VkResult;

    pub fn vkGetPhysicalDeviceFormatProperties(
        physical_device: VkPhysicalDevice,
        format: u32,
        format_properties: *mut VkFormatProperties
    );

    pub fn vkGetPhysicalDeviceProperties(
        physical_device: VkPhysicalDevice, 
        properties: *mut VkPhysicalDeviceProperties
    ) -> VkResult;

    pub fn vkGetPhysicalDeviceSurfaceSupportKHR(
        physical_device: VkPhysicalDevice,
        queue_family: u32,
        surface: VkSurfaceKHR,
        physical_device_surface_supports_KHR: *mut VkBool32
    ) -> VkResult;

    pub fn vkGetPhysicalDeviceQueueFamilyProperties(
        physical_device: VkPhysicalDevice,
        queue_family_properties_count: *mut u32, 
        queue_family_properties: *mut VkQueueFamilyProperties
    ) -> c_void;

    pub fn vkEnumerateDeviceExtensionProperties(
        physical_device: VkPhysicalDevice,
        layer_name: *const c_char,
        property_count: *mut u32,
        properties: *mut VkExtensionProperties
    ) -> VkResult;
}


#[repr(C)]
pub struct VkExtensionProperties {
    pub extension_name: [c_char; 256],
    spec_version: u32
}

#[repr(C)]
pub struct VkQueueFamilyProperties {
    pub flags: u32,
    pub count: u32,
    timestamp_valid_bits: u32,
    min_image_transfer_granulatity: c_void
}

#[repr(C)]
pub struct VkPhysicalDeviceSparseProperties {
    residency_standard2_d_block_shape: u32,
    residency_standard2_d_multisample_block_shape: u32,
    residency_standard3_d_block_shape: u32,
    residency_aligned_mip_size: u32,
    residency_non_resident_strict: u32,
}

#[repr(C)]
pub struct VkFormatProperties {
    pub linear_tiling_features: u32,
    pub optimal_tiling_features: u32,
    pub buffer_features: u32
}

#[repr(C)]
pub struct VkPhysicalDeviceProperties {
    pub api_version: u32,
    pub driver_version: u32,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: VkPhysicalDeviceType,
    pub device_name: [c_char; 256],
    pipeline_cache_uuid: [u8; 16],
    pub limits: VkPhysicalDeviceLimits,
    sparse_properties: VkPhysicalDeviceSparseProperties
}
#[repr(C)]
pub struct VkPhysicalDeviceLimits {
    max_image_dimension1_d: u32,
    max_image_dimension2_d: u32,
    max_image_dimension3_d: u32,
    max_image_dimension_cube: u32,
    max_image_array_layers: u32,
    max_texel_buffer_elements: u32,
    max_uniform_buffer_range: u32,
    max_storage_buffer_range: u32,
    max_push_constants_size: u32,
    max_memory_allocation_count: u32,
    max_sampler_allocation_count: u32,
    buffer_image_granularity: u64,
    sparse_address_space_size: u64,
    max_bound_descriptor_sets: u32,
    max_per_stage_descriptor_samplers: u32,
    max_per_stage_descriptor_uniform_buffers: u32,
    max_per_stage_descriptor_storage_buffers: u32,
    max_per_stage_descriptor_sampled_images: u32,
    max_per_stage_descriptor_storage_images: u32,
    max_per_stage_descriptor_input_attachments: u32,
    max_per_stage_resources: u32,
    max_descriptor_set_samplers: u32,
    max_descriptor_set_uniform_buffers: u32,
    max_descriptor_set_uniform_buffers_dynamic: u32,
    max_descriptor_set_storage_buffers: u32,
    max_descriptor_set_storage_buffers_dynamic: u32,
    max_descriptor_set_sampled_images: u32,
    max_descriptor_set_storage_images: u32,
    max_descriptor_set_input_attachments: u32,
    max_vertex_input_attributes: u32,
    max_vertex_input_bindings: u32,
    max_vertex_input_attribute_offset: u32,
    max_vertex_input_binding_stride: u32,
    max_vertex_output_components: u32,
    max_tessellation_generation_level: u32,
    max_tessellation_patch_size: u32,
    max_tessellation_control_per_vertex_input_components: u32,
    max_tessellation_control_per_vertex_output_components: u32,
    max_tessellation_control_per_patch_output_components: u32,
    max_tessellation_control_total_output_components: u32,
    max_tessellation_evaluation_input_components: u32,
    max_tessellation_evaluation_output_components: u32,
    max_geometry_shader_invocations: u32,
    max_geometry_input_components: u32,
    max_geometry_output_components: u32,
    max_geometry_output_vertices: u32,
    max_geometry_total_output_components: u32,
    max_fragment_input_components: u32,
    max_fragment_output_attachments: u32,
    max_fragment_dual_src_attachments: u32,
    max_fragment_combined_output_resources: u32,
    max_compute_shared_memory_size: u32,
    max_compute_work_group_count: [u32; 3],
    max_compute_work_group_invocations: u32,
    max_compute_work_group_size: [u32; 3],
    sub_pixel_precision_bits: u32,
    sub_texel_precision_bits: u32,
    mipmap_precision_bits: u32,
    max_draw_indexed_index_value: u32,
    max_draw_indirect_count: u32,
    max_sampler_lod_bias: f32,
    max_sampler_anisotropy: f32,
    max_viewports: u32,
    max_viewport_dimensions: [u32; 2],
    viewport_bounds_range: [f32; 2],
    viewport_sub_pixel_bits: u32,
    min_memory_map_alignment: usize,
    min_texel_buffer_offset_alignment: u64,
    min_uniform_buffer_offset_alignment: u64,
    min_storage_buffer_offset_alignment: u64,
    min_texel_offset: i32,
    max_texel_offset: u32,
    min_texel_gather_offset: i32,
    max_texel_gather_offset: u32,
    min_interpolation_offset: f32,
    max_interpolation_offset: f32,
    sub_pixel_interpolation_offset_bits: u32,
    max_framebuffer_width: u32,
    max_framebuffer_height: u32,
    max_framebuffer_layers: u32,
    framebuffer_color_sample_counts: u32,
    framebuffer_depth_sample_counts: u32,
    framebuffer_stencil_sample_counts: u32,
    framebuffer_no_attachments_sample_counts: u32,
    max_color_attachments: u32,
    sampled_image_color_sample_counts: u32,
    sampled_image_integer_sample_counts: u32,
    sampled_image_depth_sample_counts: u32,
    sampled_image_stencil_sample_counts: u32,
    storage_image_sample_counts: u32,
    max_sample_mask_words: u32,
    timestamp_compute_and_graphics: u32,
    timestamp_period: f32,
    max_clip_distances: u32,
    max_cull_distances: u32,
    max_combined_clip_and_cull_distances: u32,
    discrete_queue_priorities: u32,
    point_size_range: [f32; 2],
    line_width_range: [f32; 2],
    point_size_granularity: f32,
    line_width_granularity: f32,
    strict_lines: u32,
    standard_sample_locations: u32,
    optimal_buffer_copy_offset_alignment: u64,
    optimal_buffer_copy_row_pitch_alignment: u64,
    non_coherent_atom_size: u64,
}
