use crate::vulkan::{
    vertex::{
        INDICES,
        VERTICES,
        VERTEX_SIZE,
        Vertex,
        VkBuffer,
        VkBufferCopy,
        VkDeviceMemory,
        VkBufferCreateInfo,
        VkMemoryRequirements,
        VkMemoryAllocateInfo,
        VkPhysicalDeviceMemoryProperties,
        vkMapMemory,
        vkFreeMemory,
        vkUnmapMemory,
        vkCreateBuffer,
        vkDestroyBuffer,
        vkAllocateMemory,
        vkBindBufferMemory,
        vkGetBufferMemoryRequirements,
        vkGetPhysicalDeviceMemoryProperties
    },
    commands::{
        command_buffer::{
            VkCommandBufferBeginInfo,
            VkCommandBufferAllocateInfo,
            vkEndCommandBuffer,
            vkBeginCommandBuffer,
            vkFreeCommandBuffers,
            vkAllocateCommandBuffers,
        },
        vkCmdCopyBuffer
    },
    rendering::{
        VkSubmitInfo,
        vkQueueSubmit,
        vkQueueWaitIdle
    },
};

use std::ffi::{
    c_void
};


impl crate::engine::Engine { pub fn create_vertex_buffer(&mut self) {
    (self.vertex_buffer, self.vertex_buffer_memory) = self.create_device_local_buffer_with_data::<u32, _>(0x00000080, &VERTICES);
}}

impl crate::engine::Engine { pub fn create_index_buffer(&mut self) {
    (self.index_buffer, self.index_buffer_memory) = self.create_device_local_buffer_with_data::<u16, _>(0x00000040, &INDICES);
}}

impl crate::engine::Engine { pub fn create_device_local_buffer_with_data<A, T: Copy>(&self, usage: u32, data: &[T]) -> (u64, u64) {
    let mut buffer_size = (data.len() * std::mem::size_of::<T>()) as u64;

    let (staging_buffer, staging_memory, staging_size) = self.create_buffer(buffer_size, 0x00000001, 0x00000002 | 0x00000004);

    let mut data_ptr: *mut Vertex = std::ptr::null_mut();

    unsafe {vkMapMemory(
        self.device,
        staging_memory,
        0,
        buffer_size,
        0,
        &mut data_ptr as *mut *mut Vertex
    )};

    let vertex_align = std::mem::align_of::<A>();

    let layout = std::alloc::Layout::from_size_align(staging_size as usize, vertex_align).unwrap();

    unsafe {std::ptr::copy_nonoverlapping(data.as_ptr(), data_ptr as _, data.len())};

    unsafe {vkUnmapMemory(self.device, staging_memory)};


    let (buffer, memory, _) = self.create_buffer(buffer_size, 0x00000002 | usage, 0x00000001);

    self.copy_buffer(staging_buffer, buffer, buffer_size);


    unsafe {
        vkDestroyBuffer(self.device, staging_buffer, std::ptr::null());

        vkFreeMemory(self.device, staging_memory, std::ptr::null());
    };

    return (buffer, memory)
}}

impl crate::engine::Engine { pub fn create_buffer(
    &self, buffer_size: u64, usage_flags: u32, property_flags: u32
) -> (VkBuffer, VkDeviceMemory, u64) {

    let create_info = VkBufferCreateInfo {
        s_type: 12,
        p_next: std::ptr::null(),
        flags: 0,
        size: buffer_size,
        usage: usage_flags,
        sharing_mode: 0,
        queue_family_index_count: 0,
        queue_families: std::ptr::null()
    };

    let mut buffer = 0;

    unsafe {vkCreateBuffer(self.device, &create_info as *const VkBufferCreateInfo, std::ptr::null(), &mut buffer)};


    let mut memory_requirements: VkMemoryRequirements = unsafe {std::mem::zeroed()};

    unsafe {vkGetBufferMemoryRequirements(self.device, buffer, &mut memory_requirements as *mut VkMemoryRequirements)};
    

    let mut memory_properties: VkPhysicalDeviceMemoryProperties = unsafe {std::mem::zeroed()};

    unsafe {vkGetPhysicalDeviceMemoryProperties(self.physical_device, &mut memory_properties as *mut VkPhysicalDeviceMemoryProperties)};

    
    let memory_type = |memory_properties: VkPhysicalDeviceMemoryProperties, memory_requirements: &VkMemoryRequirements, property_flags: u32| -> u32 {
        for i in 0 .. memory_properties.memory_type_count {
            println!("{:?}, {:?}, {:?}", i, memory_requirements.memory_type_bits & (i << i), memory_properties.memory_types[i as usize].flags & (0x00000004 | 0x00000002));
            if 
                memory_requirements.memory_type_bits & (1 << i) != 0
                && memory_properties.memory_types[i as usize].flags & (property_flags) != 0 
            {
                return i;
            }
        }
        panic!("no memory_type found!"); return 0;
    }(memory_properties, &memory_requirements, property_flags);

    
    let allocate_info = VkMemoryAllocateInfo {
        s_type: 5,
        p_next: std::ptr::null(),
        allocation_size: memory_requirements.size,
        memory_type_index: memory_type
    };

    let mut memory = 0;

    unsafe {vkAllocateMemory(self.device, &allocate_info as *const VkMemoryAllocateInfo, std::ptr::null(), &mut memory)};


    unsafe {vkBindBufferMemory(self.device, buffer, memory, 0)};

    return (buffer, memory, memory_requirements.size)
}}

impl crate::engine::Engine { pub fn copy_buffer(&self, src: VkBuffer, dst: VkBuffer, size: u64) {
    let allocation_info = VkCommandBufferAllocateInfo {
        s_type: 40,
        p_next: std::ptr::null(),
        command_pool: self.transient_command_pool,
        level: 0,
        command_buffer_count: 1
    };

    let mut command_buffers = [0];

    unsafe {vkAllocateCommandBuffers(self.device, &allocation_info as *const VkCommandBufferAllocateInfo, command_buffers.as_mut_ptr())};

    
    let begin_info = VkCommandBufferBeginInfo {
        s_type: 42,
        p_next: std::ptr::null(),
        flags: 0x00000001,
        inheritance_info: std::ptr::null()
    };

    unsafe {vkBeginCommandBuffer(command_buffers[0], &begin_info as *const VkCommandBufferBeginInfo)};

    
    let region = VkBufferCopy {
        src_offset: 0,
        dst_offset: 0,
        size: size
    };

    let regions = [region];

    unsafe {vkCmdCopyBuffer(command_buffers[0], src, dst, regions.len() as u32, regions.as_ptr())};


    unsafe {vkEndCommandBuffer(command_buffers[0])};
    
    
    let mut submit_info: VkSubmitInfo = unsafe {std::mem::zeroed()};

    submit_info.s_type = 4;

    submit_info.command_buffer_count = command_buffers.len() as u32;
    submit_info.command_buffers = command_buffers.as_ptr();

    let submit_infos = [submit_info];

    unsafe {vkQueueSubmit(self.graphics_queue, submit_infos.len() as u32, submit_infos.as_ptr(), 0)};


    unsafe {vkQueueWaitIdle(self.graphics_queue)};


    unsafe {vkFreeCommandBuffers(self.device, self.transient_command_pool, command_buffers.len() as u32, command_buffers.as_ptr())};
}}
