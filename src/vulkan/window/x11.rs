use crate::vulkan::{
    instance::{
        VkInstance
    },
    window::{
        VkSurfaceKHR
    },
    VkResult
};

use std::ffi::{CString, c_void, c_char};


pub const CW_EVENT_MASK: u64 = 1 << 11;
pub const EXPOSURE_MASK: i64 = 0x0001 | 0x0002 | 0x0004 | 0x0008 | 0x0010 | 0x2000;


pub struct XWindow {
    pub handle: u64,
    pub root_handle: u64,
    pub display: *mut c_void,
    pub attributes: XWindowCreateAttributes
}


#[repr(C)]
pub struct XWindowCreateAttributes {
    pub background_pixmap: u64,
    pub background_pixel: u64,
    pub border_pixmap: u64,
    pub border_pixel: u64,
    pub bit_gravity: i32,
    pub win_gravity: i32,
    pub backing_store: i32,
    pub backing_planes: u64,
    pub backing_pixel: u64,
    pub save_under: i32,
    pub event_mask: i64,
    pub do_not_propagate_mask: i64,
    pub override_redirect: i32,
    pub colormap: u64,
    pub cursor: u64,
}

#[repr(C)]
pub struct XEvent {
    pub r#type: u64,
    // other fields as needed
}

#[repr(C)]
pub struct VkXlibSurfaceCreateInfoKHR {
    pub s_type: super::super::VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub dpy: *mut c_void,
    pub window: u64
}

#[link(name="vulkan")]
extern "C" {
    pub fn vkCreateXlibSurfaceKHR(
        instance: VkInstance,
        create_info: *const VkXlibSurfaceCreateInfoKHR,
        _: *const c_void,
        surface_khr: *mut VkSurfaceKHR
    ) -> VkResult;
}

#[link(name = "X11")]
extern "C" {
    pub fn XOpenDisplay(display_name: *const c_char) -> *mut c_void;
    pub fn XDefaultScreen(display: *mut c_void) -> i32;
    pub fn XRootWindow(display: *mut c_void, screen_number: i32) -> u64;
    pub fn XCreateWindow(
        display: *mut c_void,
        parent: u64,
        x: i32,
        y: i32,
        width: u32,
        height: u32,    
        border_width: u32,
        depth: i32,
        class: u32,
        visual: *mut c_void,
        valuemask: u64,
        attributes: *const XWindowCreateAttributes,
    ) -> u64;
    pub fn XStoreName(display: *mut c_void, window: u64, window_name: *const c_char);
    pub fn XMapWindow(display: *mut c_void, window: u64);
    pub fn XDestroyWindow(display: *mut c_void, window: u64);
    pub fn XNextEvent(display: *mut c_void, event: *mut XEvent);
    pub fn XSelectInput(display: *mut c_void, window: u64, event_mask: i64);
    pub fn XCloseDisplay(display: *mut c_void) -> i32;
}
