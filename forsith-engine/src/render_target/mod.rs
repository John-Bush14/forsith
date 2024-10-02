pub mod windows;


pub trait RenderTarget {
    fn create_surface_khr(&mut self);
}
