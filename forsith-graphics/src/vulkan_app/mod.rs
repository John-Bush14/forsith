use bindings::instance::VkInstance;


pub(crate) mod creation;

mod drop;


#[allow(dead_code)]
pub struct VulkanApp {
    instance: VkInstance,
}
