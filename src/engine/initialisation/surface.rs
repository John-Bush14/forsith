use winit::{dpi::LogicalSize, EventsLoop, Window, WindowBuilder};
use ash::{Entry, Instance, Device};
use std::{os::raw::c_void, ptr};
use ash::{version::{EntryV1_0, InstanceV1_0}, vk, extensions::khr::Surface};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub const WIDTH: f64 = 800.0;
pub const HEIGHT: f64 = 600.0;

pub fn create_window(name: String) -> (Window, EventsLoop) {
    let events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_title(name)
        .with_dimensions(LogicalSize::new(WIDTH, HEIGHT))
        .build(&events_loop)
        .unwrap();
    
    return (window, events_loop)
}

pub unsafe fn create_surface(
    window: &Window,
    entry: Entry,
    instance: Instance,
)

-> Result<(Surface, vk::SurfaceKHR), vk::Result>  

where Entry: EntryV1_0, Instance: InstanceV1_0, Window: HasRawWindowHandle

{
    let surface = Surface::new(&entry, &instance);
    
    let raw_window_handle = window.raw_window_handle();
    let raw_display_handle = window.raw_display_handle();

    // Determine the platform (X11 or Wayland) and create the surface accordingly
    let surface_khr = match (raw_window_handle, raw_display_handle) {
        (RawWindowHandle::Xlib(handle), RawDisplayHandle::Xlib(display)) => {
            let xlib_surface_create_info = vk::XlibSurfaceCreateInfoKHR {
                s_type: vk::StructureType::XLIB_SURFACE_CREATE_INFO_KHR,
                p_next: std::ptr::null(),
                flags: vk::XlibSurfaceCreateFlagsKHR::empty(),
                dpy: display.display as *mut _,
                window: handle.window,
            };

            surface.create_xlib_surface_khr(&xlib_surface_create_info, None)?
        },
        (RawWindowHandle::Wayland(handle), RawDisplayHandle::Wayland(display)) => {
            let wayland_surface_create_info = vk::WaylandSurfaceCreateInfoKHR {
                s_type: vk::StructureType::WAYLAND_SURFACE_CREATE_INFO_KHR,
                p_next: std::ptr::null(),
                flags: vk::WaylandSurfaceCreateFlagsKHR::empty(),
                display: display.display,
                surface: handle.surface,
            };

            surface.create_wayland_surface_khr(&wayland_surface_create_info, None)?
        },
        _ => {
            return Err(vk::Result::ERROR_INITIALIZATION_FAILED);
        }
    };
    
    return Ok((surface, surface_khr))
}
