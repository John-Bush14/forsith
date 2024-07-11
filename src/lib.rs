pub mod globals;
pub use globals::*;

//pub mod vulkan;
//pub use vulkan::*;

pub mod drawables;

pub mod engine;

pub fn initialize(name: String, version: [u8;3], event_loop: fn(i8)) {} // placeholder

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template() {
    }
}
