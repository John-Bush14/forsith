use crate::VkSurfaceKHR;


pub trait Window {
    fn initialize_connection() -> Self;

    fn map_window(&mut self);

    fn create_surface_khr(self) -> VkSurfaceKHR;
}
