pub mod globals;
pub use globals::*;

pub mod vulkan;
pub use vulkan::*;

pub mod drawables;

pub mod engine;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template() {
        vulkan::CreateDevice();
    }
}
