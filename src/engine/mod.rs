mod initialisation;
use initialisation::{};

mod drop;

#[derive(Clone)]
struct Engine {
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
