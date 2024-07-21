use crate::vulkan::{
    window::{
        WindowEvent
    }
};


impl super::Engine { pub fn start_loop(&mut self) {
    while true {
        let events = self.window.get_events();

        for event in events {match event {
            WindowEvent::Death => break,
            WindowEvent::KeyDown(keycode) => panic!(),
            _ => {}
        }}
    }
}}
