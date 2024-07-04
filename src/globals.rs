use super::drawables::Drawable;

pub struct Settings {
    pub target_fps: u16,
    pub limit_fps: bool,
}

pub struct State {
    pub fps: u16,
    pub delta: f32,
    pub drawables: Vec<Box<dyn Drawable>>
}

pub struct Flags {
    pub recalculate_vertices: bool
}

pub static mut SETTINGS: Settings = Settings {
    target_fps: 60,
    limit_fps: false
};

pub static mut STATE: State = State {
    fps: 0,
    delta: 0.0,
    drawables: vec!()
};

pub static mut FLAGS: Flags = Flags {
    recalculate_vertices: true                // "re"calculate everything at startup
};
