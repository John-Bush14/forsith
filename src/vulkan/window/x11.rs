use crate::vulkan::{
    instance::{
        VkInstance
    },
    window::{
        VkSurfaceKHR
    },
    devices::{
        physical_device::{
            VkPhysicalDevice
        }
    },
    VkResult,
    VkBool32
};

use std::ffi::{CString, c_void, c_char};
use std::fmt;

pub const CW_EVENT_MASK: u64 = 1 << 11;
pub const EXPOSURE_MASK: i64 = 0x0001 | 0x0002 | 0x0004 | 0x0008 | 0x0010 | 0x2000;


type Window = u64;

type Time = u64;

pub struct XWindow {
    pub handle: u64,
    pub root_handle: u64,
    pub display: *mut c_void,
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
    pub save_under: bool,
    pub event_mask: i64,
    pub do_not_propagate_mask: i64,
    pub override_redirect: bool,
    pub colormap: u64,
    pub cursor: u64,
}

#[repr(C)]
pub struct XWindowAttributes {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub border_width: i32,
    pub depth: i32,
    pub visual: *mut XVisual,
    pub root: Window,
    pub class: i32,
    pub bit_gravity: i32,
    pub win_gravity: i32,
    pub backing_store: i32,
    pub backing_planes: u64,
    pub backing_pixel: u64,
    pub save_under: bool,
    pub colormap: u64,
    pub map_installed: bool,
    pub map_state: i32,
    pub all_event_masks: i64,
    pub your_event_mask: i64,
    pub do_not_propagate_mask: i64,
    pub override_redirect: bool,
    pub screen: *mut c_void,
}

#[repr(C)]
pub union XEvent {
    pub type_: i32,
    pub button: XButtonEvent,
    pub key: XKeyEvent,
    pub error: XErrorEvent,
    pub destroy_window: XDestroyWindowEvent
    // other fields as needed
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XButtonEvent {
    pub type_: i32,
    pub serial: u64,
    pub send_event: bool,
    pub display: *mut c_void,
    pub window: Window,
    pub root: Window,
    pub subwindow: Window,
    pub time: Time,
    pub x: i32,
    pub y: i32,
    pub x_root: i32,
    pub y_root: i32,
    pub state: u32,
    pub button: u32,
    pub same_screen: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XDestroyWindowEvent {
    pub type_: i32,
    pub serial: u64,
    pub send_event: bool,
    pub display: *mut c_void,
    pub event: Window,
    pub window: Window,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XErrorEvent {
    pub type_: i32,
    pub display: *mut c_void,
    pub resourceid: u64, //XID
    pub serial: u64,
    pub error_code: u8,
    pub request_code: u8,
    pub minor_code: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XKeyEvent {
    pub type_: i32,
    pub serial: u64,
    pub send_event: bool,
    pub display: *mut c_void,
    pub window: Window,
    pub root: Window,
    pub subwindow: Window,
    pub time: Time,
    pub x: i32,
    pub y: i32,
    pub x_root: i32,
    pub y_root: i32,
    pub state: u32,
    pub keycode: u32,
    pub same_screen: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XVisual {
    pub ext_data: *mut c_void,
    pub visualid: u64,
    pub class: i32,
    pub red_mask: u64,
    pub green_mask: u64,
    pub blue_mask: u64,
    pub bits_per_rgb: i32,
    pub map_entries: i32,
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
    pub fn vkGetPhysicalDeviceXlibPresentationSupportKHR(
        physical_device: VkPhysicalDevice,
        queue_family_index: u32,
        display: *const c_void,
        visualid: u64
    ) -> VkBool32;
}

#[link(name = "X11")]
extern "C" {
    pub fn XOpenDisplay(display_name: *const c_char) -> *mut c_void;
    pub fn XDefaultScreen(display: *mut c_void) -> i32;
    pub fn XRootWindow(display: *mut c_void, screen_number: i32) -> Window;
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
        visual: *mut XVisual,
        valuemask: u64,
        attributes: *const XWindowCreateAttributes,
    ) -> Window;
    pub fn XStoreName(display: *mut c_void, window: u64, window_name: *const c_char);
    pub fn XMapWindow(display: *mut c_void, window: u64);
    pub fn XDestroyWindow(display: *mut c_void, window: u64);
    pub fn XNextEvent(display: *mut c_void, event: *mut XEvent) -> i32;
    pub fn XSelectInput(display: *mut c_void, window: u64, event_mask: i64);
    pub fn XCloseDisplay(display: *mut c_void) -> i32;
    pub fn XCheckMaskEvent(display: *mut c_void, mask: u32, event: *mut u64) -> u32;
    pub fn XPending(display: *mut c_void) -> i32;
    pub fn XGetWindowAttributes(display: *mut c_void, window: u64, attributes: *mut XWindowAttributes) -> i32;
}

pub const KeyPress: i32 = 2;
pub const KeyRelease: i32 = 3;
pub const ButtonPress: i32 = 4;
pub const ButtonRelease: i32 = 5;
pub const MotionNotify: i32 = 6;
pub const EnterNotify: i32 = 7;
pub const LeaveNotify: i32 = 8;
pub const FocusIn: i32 = 9;
pub const FocusOut: i32 = 10;
pub const KeymapNotify: i32 = 11;
pub const Expose: i32 = 12;
pub const GraphicsExpose: i32 = 13;
pub const NoExpose: i32 = 14;
pub const VisibilityNotify: i32 = 15;
pub const CreateNotify: i32 = 16;
pub const DestroyNotify: i32 = 17;
pub const UnmapNotify: i32 = 18;
pub const MapNotify: i32 = 19;
pub const MapRequest: i32 = 20;
pub const ReparentNotify: i32 = 21;
pub const ConfigureNotify: i32 = 22;
pub const ConfigureRequest: i32 = 23;
pub const GravityNotify: i32 = 24;
pub const ResizeRequest: i32 = 25;
pub const CirculateNotify: i32 = 26;
pub const CirculateRequest: i32 = 27;
pub const PropertyNotify: i32 = 28;
pub const SelectionClear: i32 = 29;
pub const SelectionRequest: i32 = 30;
pub const SelectionNotify: i32 = 31;
pub const ColormapNotify: i32 = 32;
pub const ClientMessage: i32 = 33;
pub const MappingNotify: i32 = 34;
pub const GenericEvent: i32 = 35;
pub const LASTEvent: i32 = 36;


impl fmt::Display for XEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = unsafe {
            match self.type_ {
                KeyPress => "key",
                KeyRelease => "key",
                ButtonPress => "button",
                ButtonRelease => "button",
                MotionNotify => "motion", 
                EnterNotify => "crossing", 
                LeaveNotify => "crossing", 
                FocusIn => "focus_change", 
                FocusOut => "focus_change",
                KeymapNotify => "keymap", 
                Expose => "expose", 
                GraphicsExpose => "graphics_expose",
                NoExpose => "no_expose",
                VisibilityNotify => "visibility",
                CreateNotify => "create_window",
                DestroyNotify => "destroy_window",
                UnmapNotify => "unmap",
                MapNotify => "map",
                MapRequest => "map_request",
                ReparentNotify => "reparent",
                ConfigureNotify => "configure",
                ConfigureRequest => "configure_request",
                GravityNotify => "gravity",
                ResizeRequest => "resize_request",
                CirculateNotify => "circulate",
                CirculateRequest => "circulate_request",
                PropertyNotify => "property",
                SelectionClear => "selection_clear",
                SelectionRequest => "selection_request",
                SelectionNotify => "selection",
                ColormapNotify => "colormap",
                ClientMessage => "client_message",
                MappingNotify => "mapping",
                GenericEvent => "generic_event_cookie",
                _ => "any"
            }
        };

        write!(f, "event: {}", string); return Ok(());
    }
}
