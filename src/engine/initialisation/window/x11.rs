use crate::vulkan::{
    window::{
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
            XWindowCreateAttributes,
            XEvent,
            XWindow,
            VkXlibSurfaceCreateInfoKHR,
            vkCreateXlibSurfaceKHR
        },
        Window,
        VkSurfaceKHR,
        WindowEvent
    },
    instance::{
        VkInstance
    }
};

use std::ffi::{
    CString,
    c_void
};


const EXPOSURE_MASK: i32 = 0;
const CW_EVENT_MASK: u64 = 0;


impl Window for XWindow {
    fn start_loop(&self, function: fn(event: WindowEvent)) {
        while true {
            let mut event: XEvent = unsafe {std::mem::zeroed()};
            unsafe {XNextEvent(self.display, &mut event)};

            function(WindowEvent::test);
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

    fn init(name: String) -> XWindow { unsafe {
        let display = XOpenDisplay(std::ptr::null()) as *mut c_void;  
        if display.is_null() {panic!("XOpenDisplay failed! :'(");}

        let screen_number = XDefaultScreen(display);
        let root_window = XRootWindow(display, screen_number);
    
        let mut window_attributes: XWindowCreateAttributes = std::mem::zeroed();
        window_attributes.background_pixel = 0;
        window_attributes.event_mask = EXPOSURE_MASK as i64;

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

        let window_title = CString::new(name).expect("CString::new failed");
        XStoreName(display, window, window_title.as_ptr());
    
        // Map (show) the window
        XMapWindow(display, window);
    
        // Select input events to listen for
        XSelectInput(display, window, EXPOSURE_MASK as i64);
    
        // Event loop
        let mut event: XEvent = std::mem::zeroed();
        while false {
            panic!();
            XNextEvent(display, &mut event);
            match event.r#type {
                0 => {
                    XDestroyWindow(display, window);
                    XCloseDisplay(display);
                },
            
                2 => {break;}, // key down
                3 => {}, // key up
    
                4 => {}, // mouse down
                5 => {}, // mouse up
            
                6 => {} // mouse movement? not working.

                7 => {}, // startup / meta data change

                _ => {panic!("{:?}", event.r#type);}
            }
       
            println!("{:?}", event.r#type);
        }
    
        return XWindow {
            display: display,
            handle: window,
            root_handle: root_window,
            attributes: window_attributes
        };
    }}
}
