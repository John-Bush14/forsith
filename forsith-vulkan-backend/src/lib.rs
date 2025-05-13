#![allow(clippy::needless_return)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use bindings::{vk_version, VkVersion};


pub mod vulkan_app;

pub mod render_target;

//pub(crate) mod command_pool;

pub mod errors;


pub type DynError = Box<dyn std::error::Error>;


pub const API_VERSION: fn() -> VkVersion = || vk_version(1, 0, 0);
