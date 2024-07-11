use ash::{vk, Entry, Instance, Device};
use ash::{
    extensions::ext::DebugReport,
    version::{EntryV1_0, InstanceV1_0},
};

mod initialisation;
use initialisation::{};

mod drop;

#[derive(Clone)]
struct Engine {
    entry: Entry,
    instance: Instance,
    debug_report_callback: Option<(DebugReport, vk::DebugReportCallbackEXT)>,
    physical_device: vk::PhysicalDevice,
    device: Device,
    graphics_queue: vk::Queue
}

static mut ENGINE: Option<Engine> = None;

pub fn initialize_engine(name: String, version: [u8;3], event_loop: fn(i8)) {
    unsafe { ENGINE = Some(Engine::init(name, version, event_loop).expect("Initialisation of engine failed")); }
}

pub fn engine() -> Engine {unsafe {
    if ENGINE.is_none() {
        panic!("Engine hasn't be initialized yet");
    }

    return ENGINE.clone().unwrap();
}}
