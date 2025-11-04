use std::ffi::c_void;

pub enum DisplayTarget {
    Headless,
    Xorg {display: *const c_void, visual_id: u64},
    Wayland {wl_display: *const c_void}

}

pub trait GraphicsApi {
    fn connect(engine_name: &str, engine_version: (u32,u32,u32), target: DisplayTarget) -> Self;
}
