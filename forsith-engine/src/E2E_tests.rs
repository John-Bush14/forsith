/// open's up window and tries to show as many features as possible,
/// programmer is supposed to check if these features show up as test.
#[cfg(test)]
mod visual_test {
    use crate::{vk_make_version, vulkan_app::VulkanApp};

    #[test]
    fn test() {
        let _vulkan_app = VulkanApp::new("test", vk_make_version(0, 0, 0)).expect("Some error occured in VulkanApp::new()");
    }
}
