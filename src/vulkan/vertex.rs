pub const VERTICES: [Vertex; 4] = [
    Vertex {
        pos: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [0.5, -0.5],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        pos: [0.5, 0.5],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [-0.5, 0.5],
        color: [0.0, 1.0, 0.0, 1.0],
    },
];

pub const INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

pub const VERTEX_SIZE: usize = 24;


use crate::vulkan::{
    devices::{
        device::{
            VkDevice
        },
        physical_device::{
            VkPhysicalDevice
        }
    },
    VkResult, 
    VkStructureType
};

use std::ffi::{
    c_void
};


pub type VkBuffer = u64;

pub type VkDeviceMemory = u64;


#[link(name = "vulkan")]
extern "C" {
    pub fn vkCreateBuffer(
        device: VkDevice,
        create_info: *const VkBufferCreateInfo,
        _: *const c_void,
        buffer: *mut VkBuffer
    ) -> VkResult;
    
    pub fn vkGetBufferMemoryRequirements(
        device: VkDevice,
        buffer: VkBuffer,
        memory_requirements: *mut VkMemoryRequirements
    );

    pub fn vkGetPhysicalDeviceMemoryProperties(
        physical_device: VkPhysicalDevice,
        memory_properties: *mut VkPhysicalDeviceMemoryProperties
    );

    pub fn vkAllocateMemory(
        device: VkDevice,
        allocate_info: *const VkMemoryAllocateInfo,
        _: *const c_void,
        memory: *mut VkDeviceMemory
    ) -> VkResult;

    pub fn vkBindBufferMemory(
        device: VkDevice,
        buffer: VkBuffer,
        memory: VkDeviceMemory,
        memory_offset: u64
    ) -> VkResult;

    pub fn vkMapMemory(
        device: VkDevice,
        memory: VkDeviceMemory,
        offset: u64,
        size: u64,
        flags: u32,
        data: *mut *mut c_void
    ) -> VkResult;

    pub fn vkUnmapMemory(
        device: VkDevice,
        memory: VkDeviceMemory
    );

    pub fn vkDestroyBuffer(
        device: VkDevice,
        buffer: VkBuffer,
        _: *const c_void
    );

    pub fn vkFreeMemory(
        device: VkDevice,
        memory: VkDeviceMemory,
        _: *const c_void
    );
}


#[repr(C)]
pub struct VkBufferCopy {
    pub src_offset: u64,
    pub dst_offset: u64,
    pub size: u64
}

#[repr(C)]
pub struct VkMemoryAllocateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub allocation_size: u64,
    pub memory_type_index: u32
}

#[repr(C)]
pub struct VkMemoryType {
    pub flags: u32,
    pub index: u32
}

#[repr(C)]
pub struct VkMemoryHeap {
    pub size: u64,
    pub flags: u32
}

#[repr(C)]
pub struct VkPhysicalDeviceMemoryProperties {
    pub memory_type_count: u32,
    pub memory_types: [VkMemoryType; 32],
    pub memory_heap_count: u32,
    pub memory_heaps: [VkMemoryHeap; 16]
}

#[repr(C)]
pub struct VkMemoryRequirements {
    pub size: u64,
    pub alignment: u64,
    pub memory_type_bits: u32
}

#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 4]
}

impl Eq for Vertex {}

impl std::hash::Hash for Vertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for &pos_val in &self.pos {
            state.write_u32(pos_val.to_bits());
        }
        for &color_val in &self.color {
            state.write_u32(color_val.to_bits());
        }
    }
}

#[repr(C)]
pub struct VkVertexInputBindingDescription {
    pub binding: u32,
    pub stride: u32,
    pub input_rate: u32
}

#[repr(C)]
pub struct VkVertexInputAttributeDescription {
    pub location: u32,
    pub binding: u32,
    pub format: u32,
    pub offset: u32
}

#[repr(C)]
pub struct VkBufferCreateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub size: u64,
    pub usage: u32,
    pub sharing_mode: u32,
    pub queue_family_index_count: u32,
    pub queue_families: *const u32
}
