use crate::vulkan::{
    window::{
        x11,
        x11::{
            XOpenDisplay,
            XDefaultScreen,
            XRootWindow,
            XInitThreads,
            XResizeWindow,
            XCreateWindow,
            XStoreName,
            XFlush,
            XSync,
            XMapWindow,
            XWarpPointer,
            XConfigureWindow,
            XSelectInput,
            XDestroyWindow,
            XCloseDisplay,
            XNextEvent,
            XPending,
            XGrabPointer,
            XUngrabPointer,
            XWindowCreateAttributes,
            XGetWindowAttributes,
            XInternAtom,
            XSetWMProtocols,
            XConfigureEvent,
            XEvent,
            XWindow,
            VkXlibSurfaceCreateInfoKHR,
            XVisual,
            XWindowAttributes,
            Bool,
            XWindowChanges,
            XSendEvent,
            XAtom,
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


const EXPOSURE_MASK: i64 = 
    0x0002_0000     // Stucture Notify
    | 0x0020_0000   // property change mask   
    | 0x0001_0000   // visibiliy change
    | 0x0000_0001   // key down
    | 0x0000_0002   // key up
    | 0x0000_0004   // button down
    | 0x0000_0008   // button up
    | 0x0000_0040
;

const CW_EVENT_MASK: u64 = 0x0800;


impl Window for XWindow {
    fn get_events(&mut self, dimensions: [i32; 2]) -> Vec<WindowEvent> {
        let mut events: Vec<WindowEvent> = vec!();

        while unsafe{XPending(self.display)} > 0 {
            let mut event: XEvent = unsafe {std::mem::zeroed()};

            unsafe {XNextEvent(self.display, &mut event)};

            //println!("{}, {}", event, unsafe{XPending(self.display)});

            unsafe {events.push(match event.type_ {
                x11::CreateNotify => WindowEvent::Birth,
                x11::KeyRelease => WindowEvent::KeyUp(event.key.keycode),
                x11::KeyPress => WindowEvent::KeyDown(event.key.keycode),
                x11::ButtonRelease => WindowEvent::MouseUp(event.button.button),
                x11::ButtonPress => WindowEvent::MouseDown(event.button.button),
                x11::ConfigureNotify => {
                    if dimensions[0] != event.configure.width || dimensions[1] != event.configure.height {
                        WindowEvent::WindowResize([event.configure.width, event.configure.height])
                    }
                    else {WindowEvent::Undefined}
                },
                x11::MotionNotify => {
                    let x = self.mouse_position[0] - event.mouse_motion.x as f32;

                    let y = self.mouse_position[1] - event.mouse_motion.y as f32;

                    self.mouse_position = [event.mouse_motion.x as f32, event.mouse_motion.y as f32];

                    WindowEvent::MouseMove(x, y)
                }
                _ => {
                    if unsafe{event.type_} == x11::ClientMessage 
                    && unsafe {event.client_message.data.longs[0] as XAtom} == self.delete_window_protocol {
                        WindowEvent::Death
                    } else {WindowEvent::Undefined}
                }
            })};
        }

        return events
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
            window: self.handle,
        };
        
        let mut surface_KHR: VkSurfaceKHR = 0;

        unsafe { vkCreateXlibSurfaceKHR(instance, &create_info, std::ptr::null(), &mut surface_KHR); };

        return surface_KHR
    }

    fn init_connection(dimensions: [i32; 2]) -> XWindow { unsafe {
        XInitThreads();

        let display = XOpenDisplay(std::ptr::null()) as *mut c_void;  
        if display.is_null() {panic!("XOpenDisplay failed! :'(");}

        let screen_number = XDefaultScreen(display);
        let root_window = XRootWindow(display, screen_number);

         XSelectInput(display, root_window, EXPOSURE_MASK);
        
        unsafe {XWarpPointer(display, 0, root_window, 0, 0, 0, 0, 100, 100)};    
        unsafe {XFlush(display)};
    
        let mut window_attributes: XWindowCreateAttributes = std::mem::zeroed();
        window_attributes.background_pixel = 0;
        window_attributes.event_mask = EXPOSURE_MASK;

        let window = XCreateWindow(
            display,
            root_window,
            100,
            100,
            dimensions[0] as u32,
            dimensions[1] as u32,
            1,
            0,
            1,
            std::ptr::null_mut(),
            CW_EVENT_MASK,
            &window_attributes as *const XWindowCreateAttributes,
        );      

        println!("{}", window);

        return XWindow {
            display: display,
            handle: window,
            root_handle: root_window,
            delete_window_protocol: 0,
            mouse_position: [0.0, 0.0]
        };
    }}

    fn init_window(&mut self, name: String) { unsafe {
        let window_title = CString::new(name).expect("CString::new failed");
        XStoreName(self.display, self.handle, window_title.as_ptr());

        let wm_protocols_str = CString::new("WM_PROTOCOLS").unwrap();
        let wm_delete_window_str = CString::new("WM_DELETE_WINDOW").unwrap();

        let wm_protocols = XInternAtom(self.display, wm_protocols_str.as_ptr(), 0);
        let wm_delete_window = XInternAtom(self.display, wm_delete_window_str.as_ptr(), 0);

        self.delete_window_protocol = wm_delete_window;

        let mut protocols = [wm_delete_window];

        XSetWMProtocols(
            self.display,
            self.handle,
            protocols.as_mut_ptr(),
            protocols.len() as i32,
        );
    
        // Map (show) the window
        XMapWindow(self.display, self.handle);
    
        // Select input events to listen for
        XSelectInput(self.display, self.root_handle, EXPOSURE_MASK as i64);
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

    fn set_mouse(&mut self, x: f32, y: f32) {
        unsafe {XWarpPointer(self.display, 0, self.handle, 0, 0, 0, 0, x as i32, y as i32)};    
        unsafe {XFlush(self.display)};
        self.mouse_position = [x, y];
    }
}
