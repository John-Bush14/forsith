#[cfg(test)]
mod tests {
    use crate::{engine, vulkan::window::WindowEvent};


    #[test]
    fn template() {
        engine::initialize_engine(
            "test fps demo".to_string(),
            [0, 1, 0],
            (-90.0f32, 0.0f32),

            |engine, data| {},

            |engine: &mut crate::engine::Engine, (yaw, pitch)| {
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

                            *yaw += -dx * sensitivity;
                            *pitch += (dy * sensitivity).min(89.0).max(-89.0);;

                            engine.world_view.set_target_yaw_pitch(*yaw, *pitch);
                        
                            engine.window.set_mouse((engine.dimensions[0]/2) as f32, (engine.dimensions[1]/2) as f32);
                       }
                    },
                    _ => {}
                }
           }
        })
    }
}
