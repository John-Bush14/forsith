pub mod globals;
pub use globals::*;

pub mod vulkan;

pub mod drawables;

pub mod engine;

pub fn initialize(name: String, version: [u8;3], event_loop: fn()) {} 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template() {
        engine::initialize_engine("test".to_string(), [0, 0, 0], || return ())
    }
}
