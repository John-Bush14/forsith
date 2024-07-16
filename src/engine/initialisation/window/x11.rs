use crate::vulkan::{
    window::{
        x11,
        x11::{
            XOpenDisplay,
            XDefaultScreen,
            XRootWindow,
            XCreateWindow,
            XStoreName,
            XMapWindow,
            XSelectInput,
            XDestroyWindow,
            XCloseDisplay,
            XNextEvent,
            XPending,
            XWindowCreateAttributes,
            XGetWindowAttributes,
            XEvent,
            XWindow,
            VkXlibSurfaceCreateInfoKHR,
            XVisual,
            XWindowAttributes,
            vkCreateXlibSurfaceKHR,
            vkGetPhysicalDeviceXlibPresentationSupportKHR,
        },
        Window,
        VkSurfaceKHR,
        WindowEvent
    },
    instance::{
        VkInstance
    },
    devices::{
        physical_device::{
            VkPhysicalDevice
        }
    },
    VkBool32
};

use std::ffi::{
    CString,
    c_void
};


const EXPOSURE_MASK: i64 = 0xFFFFFF | 0x0002_0000;
const CW_EVENT_MASK: u64 = 0xFFFFFFFF | 0x0800;


impl Window for XWindow {
    fn start_loop(&self, function: fn()) {
        while true {
            let mut event: XEvent = unsafe {std::mem::zeroed()};
            if unsafe {XPending(self.display)} > 0 {unsafe {XNextEvent(self.display, &mut event)};}

            function();

            unsafe { if event.type_  != 0 {println!("{}", event)}};

            if unsafe{event.type_} == x11::KeyRelease {panic!()}
        }
    }

    fn get_width(&self) -> u32 {todo!();}
    fn get_height(&self) -> u32 {todo!();}

    fn set_width(&mut self, w: u32) {todo!();}
    fn set_height(&mut self, h: u32) {todo!();}

    fn create_surfaceKHR(&self, instance: VkInstance) -> VkSurfaceKHR {
        let create_info = VkXlibSurfaceCreateInfoKHR {
            s_type: 1000004000,
            p_next: std::ptr::null(),
            flags: 0,
            dpy: self.display,
            window: self.handle
        };
        
        let mut surface_KHR: VkSurfaceKHR = 0;

        unsafe { vkCreateXlibSurfaceKHR(instance, &create_info, std::ptr::null(), &mut surface_KHR); };

        return surface_KHR
    }

    fn init_connection() -> XWindow { unsafe {
        let display = XOpenDisplay(std::ptr::null()) as *mut c_void;  
        if display.is_null() {panic!("XOpenDisplay failed! :'(");}

        let screen_number = XDefaultScreen(display);
        let root_window = XRootWindow(display, screen_number);
    
        let mut window_attributes: XWindowCreateAttributes = std::mem::zeroed();
        window_attributes.background_pixel = 0;
        window_attributes.event_mask = EXPOSURE_MASK;

        let mut visual: XVisual = std::mem::zeroed();

        let window = XCreateWindow(
            display,
            root_window,
            100,
            100,
            800,
            600,
            1,
            0,
            1,
            std::ptr::null_mut(),
            CW_EVENT_MASK,
            &window_attributes as *const XWindowCreateAttributes,
        );      

        return XWindow {
            display: display,
            handle: window,
            root_handle: root_window,
        };
    }}

    fn init_window(&self, name: String) { unsafe {
        let window_title = CString::new(name).expect("CString::new failed");
        XStoreName(self.display, self.handle, window_title.as_ptr());
    
        // Map (show) the window
        XMapWindow(self.display, self.handle);
    
        // Select input events to listen for
        XSelectInput(self.display, self.handle, EXPOSURE_MASK as i64);
    }}
    
    fn supports_physical_device_queue(&self, physical_device: VkPhysicalDevice, queue: u32) -> bool {
        let mut attributes: XWindowAttributes = unsafe {std::mem::zeroed()};

        unsafe {XGetWindowAttributes(self.display, self.handle, &mut attributes as *mut XWindowAttributes)};

        let support: VkBool32 = unsafe { vkGetPhysicalDeviceXlibPresentationSupportKHR(
            physical_device, queue, self.display, (*attributes.visual).visualid
        )};

        return support == 1;
    }

    fn commit_suicide(&self) {
        unsafe { XDestroyWindow(self.display, self.handle)};

        unsafe { XCloseDisplay(self.display)};
    }
}
