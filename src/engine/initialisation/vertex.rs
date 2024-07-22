use crate::vulkan::{
    vertex::{
        VERTICES,
        VERTEX_SIZE,
        Vertex,
        VkBufferCreateInfo,
        VkMemoryRequirements,
        VkMemoryAllocateInfo,
        VkPhysicalDeviceMemoryProperties,
        vkMapMemory,
        vkUnmapMemory,
        vkCreateBuffer,
        vkAllocateMemory,
        vkBindBufferMemory,
        vkGetBufferMemoryRequirements,
        vkGetPhysicalDeviceMemoryProperties
    }
};

use std::ffi::{
    c_void
};


impl crate::engine::Engine { pub fn create_vertex_buffer(&mut self) {
    let create_info = VkBufferCreateInfo {
        s_type: 12,
        p_next: std::ptr::null(),
        flags: 0,
        size: (VERTICES.len() * VERTEX_SIZE) as u64,
        usage: 0x00000080,
        sharing_mode: 0,
        queue_family_index_count: 0,
        queue_families: std::ptr::null()
    };

    unsafe {vkCreateBuffer(self.device, &create_info as *const VkBufferCreateInfo, std::ptr::null(), &mut self.vertex_buffer)};


    let mut memory_requirements: VkMemoryRequirements = unsafe {std::mem::zeroed()};

    unsafe {vkGetBufferMemoryRequirements(self.device, self.vertex_buffer, &mut memory_requirements as *mut VkMemoryRequirements)};
    

    let mut memory_properties: VkPhysicalDeviceMemoryProperties = unsafe {std::mem::zeroed()};

    unsafe {vkGetPhysicalDeviceMemoryProperties(self.physical_device, &mut memory_properties as *mut VkPhysicalDeviceMemoryProperties)};

    
    let memory_type = |memory_properties: VkPhysicalDeviceMemoryProperties, memory_requirements: &VkMemoryRequirements| -> u32 {
        for i in 0 .. memory_properties.memory_type_count {
            println!("{:?}, {:?}, {:?}", i, memory_requirements.memory_type_bits & (i << i), memory_properties.memory_types[i as usize].flags & (0x00000004 | 0x00000002));
            if 
                memory_requirements.memory_type_bits & (1 << i) != 0
                && memory_properties.memory_types[i as usize].flags & (0x00000002 | 0x00000004) != 0 
            {
                return i;
            }
        }
        panic!("no memory_type found!"); return 0;
    }(memory_properties, &memory_requirements);

    
    let allocate_info = VkMemoryAllocateInfo {
        s_type: 5,
        p_next: std::ptr::null(),
        allocation_size: memory_requirements.size,
        memory_type_index: memory_type
    };

    unsafe {vkAllocateMemory(self.device, &allocate_info as *const VkMemoryAllocateInfo, std::ptr::null(), &mut self.vertex_buffer_memory)};


    unsafe {vkBindBufferMemory(self.device, self.vertex_buffer, self.vertex_buffer_memory, 0)};


    let mut data_ptr: *mut Vertex = std::ptr::null_mut();

    unsafe {vkMapMemory(
        self.device,
        self.vertex_buffer_memory,
        0,
        create_info.size,
        0,
        &mut data_ptr as *mut *mut Vertex
    )};

    let vertex_align = std::mem::align_of::<u32>();

    let layout = std::alloc::Layout::from_size_align(create_info.size as usize, vertex_align).unwrap();

    unsafe {std::ptr::copy_nonoverlapping(VERTICES.as_ptr(), data_ptr, VERTICES.len())};

    unsafe {vkUnmapMemory(self.device, self.vertex_buffer_memory)};
}}
