pub mod globals;
pub use globals::*;

pub use engine::interface::drawables::drawable;

pub mod vulkan;

pub mod engine;

pub fn initialize(name: String, version: [u8;3], event_loop: fn()) {} 

