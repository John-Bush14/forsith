
#[cfg(test)]
mod tests {
    use crate::{engine, vulkan::window::WindowEvent};

    struct State {
        yaw: f32,
        pitch: f32
    }


    #[test]
    fn template() {
        engine::initialize_engine(
            "test fps demo".to_string(),
            [0, 1, 0],
            State {yaw: -90.0, pitch: 0.0},

            |engine, state| {
                let drawable = crate::drawable::rect_from_transform([0.0, 0.0], 1.0, 1.0, 0.0, [0.0, 1.0, 0.0, 1.0]);
    
                engine.add_drawable(drawable);
            },

            |engine: &mut crate::engine::Engine, state| {
            for event in &engine.events {
                match event {
                    WindowEvent::KeyDown(keycode) => {
                       let mut movement = [0.0, 0.0, 0.0];

                        let speed = 0.5;

                        match *keycode {
                            65 => {panic!()},

                            25 => {movement[2] += speed}, // z
                            38 => {movement[0] -= speed}, // q
                            39 => {movement[2] -= speed}, // s
                            40 => {movement[0] += speed}  // d

                           _ => {println!("key!: {}", keycode)}
                        };

                        engine.world_view.move_eye_local(movement);
                    },
                    WindowEvent::MouseMove(dx, dy) => {
                        if (*dx, *dy) != (0.0, 0.0) {
                            let sensitivity = 0.1;

                            state.yaw += -dx * sensitivity;
                            state.pitch += (dy * sensitivity).min(89.0).max(-89.0);;

                            engine.world_view.set_target_yaw_pitch(state.yaw, state.pitch);
                        
                            engine.window.set_mouse((engine.dimensions[0]/2) as f32, (engine.dimensions[1]/2) as f32);
                       }
                    },
                    _ => {}
                }
           }
        })
    }
}
