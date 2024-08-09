use crate::vulkan::{
    instance::VkInstance,
    window::VkSurfaceKHR,
    devices::physical_device::VkPhysicalDevice,
    VkResult,
    VkBool32
};

use std::ffi::{c_void, c_char};
use std::fmt;

pub const CW_EVENT_MASK: u64 = 1 << 11;
pub const EXPOSURE_MASK: i64 = 0x0001 | 0x0002 | 0x0004 | 0x0008 | 0x0010 | 0x2000;

pub type Bool = i32;

pub type XAtom = u64;

type Window = u64;

type Time = u64;

pub struct XWindow {
    pub handle: u64,
    pub root_handle: u64,
    pub display: *mut c_void,
    pub delete_window_protocol: XAtom,
    pub mouse_position: [f32;2],
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
    pub override_redirect: Bool,
    pub colormap: u64,
    pub cursor: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub save_under: Bool,
    pub colormap: u64,
    pub map_installed: Bool,
    pub map_state: i32,
    pub all_event_masks: i64,
    pub your_event_mask: i64,
    pub do_not_propagate_mask: i64,
    pub override_redirect: Bool,
    pub screen: *mut c_void,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XConfigureEvent {
    pub type_: i32,
    pub serial: u64,
    pub send_event: Bool,
    pub display: *mut c_void,
    pub event: Window,
    pub window: Window,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub border_width: i32,
    pub above: Window,
    pub override_redirect: Bool,
}

#[repr(C)]
pub union XEvent {
    pub type_: i32,
    pub button: XButtonEvent,
    pub key: XKeyEvent,
    pub error: XErrorEvent,
    pub destroy_window: XDestroyWindowEvent,
    pub client_message: XClientMessageEvent,
    pub resize_request: XResizeRequestEvent,
    pub configure: XConfigureEvent,
    pub mouse_motion: XMotionEvent
    // other fields as needed
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XMotionEvent {
    pub type_: i32,
    pub serial: u64,
    pub send_event: Bool,
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
    pub is_hint: c_char,
    pub same_screen: Bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XResizeRequestEvent {
    pub type_: i32,
    pub serial: u64,
    pub send_event: Bool,
    pub display: *mut c_void,
    pub window: Window,
    pub width: i32,
    pub height: i32
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XButtonEvent {
    pub type_: i32,
    pub serial: u64,
    pub send_event: Bool,
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
    pub same_screen: Bool,
}

#[repr(C)]
pub struct XWindowChanges {
    x: i32,
    y: i32,
    pub width: i32,
    pub height: i32,
    sibling: Window,
    stack_mode: i32
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XDestroyWindowEvent {
    pub type_: i32,
    pub serial: u64,
    pub send_event: Bool,
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
pub struct XClientMessageEvent {
    pub type_: i32,
    pub serial: u64,
    pub send_event: Bool,
    pub display: *mut c_void,
    pub window: Window,
    pub message_type: XAtom,
    pub format: i32,
    pub data: ClientMessageData,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct ClientMessageData {
    pub longs: [i64; 5],
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XKeyEvent {
    pub type_: i32,
    pub serial: u64,
    pub send_event: Bool,
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
    pub same_screen: Bool,
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
    pub fn XMapWindow(display: *mut c_void, window: u64) -> i32;
    pub fn XDestroyWindow(display: *mut c_void, window: u64);
    pub fn XNextEvent(display: *mut c_void, event: *mut XEvent) -> i32;
    pub fn XSelectInput(display: *mut c_void, window: u64, event_mask: i64);
    pub fn XCloseDisplay(display: *mut c_void) -> i32;
    pub fn XCheckMaskEvent(display: *mut c_void, mask: u32, event: *mut u64) -> u32;
    pub fn XPending(display: *mut c_void) -> i32;
    pub fn XGetWindowAttributes(display: *mut c_void, window: u64, attributes: *mut XWindowAttributes) -> i32;
    pub fn XInternAtom(display: *mut c_void, atom_name: *const c_char, only_if_exists: Bool) -> XAtom;
    pub fn XSetWMProtocols(display: *mut c_void, window: Window, protocols: *const XAtom, count: i32);
    pub fn XInitThreads();
    pub fn XResizeWindow(display: *mut c_void, window: Window, width: u32, height: u32);
    pub fn XConfigureWindow(display: *mut c_void, window: Window, value_mask: u32, configuration: *const XWindowChanges);
    pub fn XSendEvent(display: *mut c_void, window: Window, propagate: Bool, event_mask: i64, event: *const XEvent);
    pub fn XWarpPointer(display: *mut c_void, src_w: Window, dst_w: Window, src_x: i32, src_y: i32, src_width: u32, src_height: u32, dst_x: i32, dst_y: i32) -> i32;
    pub fn XFlush(display: *mut c_void) -> i32;
    pub fn XSync(display: *mut c_void, _: Bool) -> i32;
    pub fn XGrabPointer (_9: *mut c_void, _8: u64, _7: i32, _6: u32, _5: i32, _4: i32, _3: u64, _2: u64, _1: u64) -> i32;
    pub fn XUngrabPointer(display: *mut c_void, time: u64);
}

pub const KEY_PRESS: i32 = 2;
pub const KEY_RELEASE: i32 = 3;
pub const BUTTON_PRESS: i32 = 4;
pub const BUTTON_RELEASE: i32 = 5;
pub const MOTION_NOTIFY: i32 = 6;
pub const ENTER_NOTIFY: i32 = 7;
pub const LEAVE_NOTIFY: i32 = 8;
pub const FOCUS_IN: i32 = 9;
pub const FOCUS_OUT: i32 = 10;
pub const KEYMAP_NOTIFY: i32 = 11;
pub const EXPOSE: i32 = 12;
pub const GRAPHICS_EXPOSE: i32 = 13;
pub const NO_EXPOSE: i32 = 14;
pub const VISIBILITY_NOTIFY: i32 = 15;
pub const CREATE_NOTIFY: i32 = 16;
pub const DESTROY_NOTIFY: i32 = 17;
pub const UNMAP_NOTIFY: i32 = 18;
pub const MAP_NOTIFY: i32 = 19;
pub const MAP_REQUEST: i32 = 20;
pub const REPARENT_NOTIFY: i32 = 21;
pub const CONFIGURE_NOTIFY: i32 = 22;
pub const CONFIGURE_REQUEST: i32 = 23;
pub const GRAVITY_NOTIFY: i32 = 24;
pub const RESIZE_REQUEST: i32 = 25;
pub const CIRCULATE_NOTIFY: i32 = 26;
pub const CIRCULATE_REQUEST: i32 = 27;
pub const PROPERTY_NOTIFY: i32 = 28;
pub const SELECTION_CLEAR: i32 = 29;
pub const SELECTION_REQUEST: i32 = 30;
pub const SELECTION_NOTIFY: i32 = 31;
pub const COLORMAP_NOTIFY: i32 = 32;
pub const CLIENT_MESSAGE: i32 = 33;
pub const MAPPING_NOTIFY: i32 = 34;
pub const GENERIC_EVENT: i32 = 35;
pub const LASTEVENT: i32 = 36;


impl fmt::Display for XEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = unsafe {
            match self.type_ {
                KEY_PRESS => "key",
                KEY_RELEASE => "key",
                BUTTON_PRESS => "button",
                BUTTON_RELEASE => "button",
                MOTION_NOTIFY => "motion", 
                ENTER_NOTIFY => "crossing", 
                LEAVE_NOTIFY => "crossing", 
                FOCUS_IN => "focus_change", 
                FOCUS_OUT => "focus_change",
                KEYMAP_NOTIFY => "keymap", 
                EXPOSE => "expose", 
                GRAPHICS_EXPOSE => "graphics_expose",
                NO_EXPOSE => "no_expose",
                VISIBILITY_NOTIFY => "visibility",
                CREATE_NOTIFY => "create_window",
                DESTROY_NOTIFY => "destroy_window",
                UNMAP_NOTIFY => "unmap",
                MAP_NOTIFY => "map",
                MAP_REQUEST => "map_request",
                REPARENT_NOTIFY => "reparent",
                CONFIGURE_NOTIFY => "configure",
                CONFIGURE_REQUEST => "configure_request",
                GRAVITY_NOTIFY => "gravity",
                RESIZE_REQUEST => "resize_request",
                CIRCULATE_NOTIFY => "circulate",
                CIRCULATE_REQUEST => "circulate_request",
                PROPERTY_NOTIFY => "property",
                SELECTION_CLEAR => "selection_clear",
                SELECTION_REQUEST => "selection_request",
                SELECTION_NOTIFY => "selection",
                COLORMAP_NOTIFY => "colormap",
                CLIENT_MESSAGE => "client_message",
                MAPPING_NOTIFY => "mapping",
                GENERIC_EVENT => "generic_event_cookie",
                _ => "any"
            }
        };

        let _ = write!(f, "event: {}", string); return Ok(());
    }
}
