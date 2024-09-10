
#[cfg(test)]
mod tests {
    use crate::{engine, vulkan::{pipeline::{ShaderItem, ShaderStage}, window::WindowEvent}, Drawable, PIPELINE_UI_IMAGE_2D, PIPELINE_UI_IMAGE_3D};

    struct State {
        yaw: f32,
        pitch: f32,
        momentum: [f32;3]
    }

    #[test]
    fn template() {
        engine::initialize_engine(
            "test fps demo".to_string(),
            [0, 1, 0], // version

            |engine| {
                engine.target_fps = 60.0;

                let mut rect = Drawable::rect_from_transform([-0.5, -0.5], 0.25, 0.25, 0.0, [1.0;4]);
                rect.set_pipeline_id(PIPELINE_UI_IMAGE_2D);

                rect.prepare_uniforms(&engine.pipelines);

                let image = engine.create_texture("src/engine/assets/test.ppm".to_string(), false);
                *rect.get_uniform(ShaderStage::Fragment, 0) = ShaderItem::Sampler2D(image);

                let among_us = "src/engine/assets/among_us/among_us.obj";
                let among_us_texture = engine.create_texture("src/engine/assets/among_us/among_us.ppm".to_string(), true);
                let mut model = Drawable::model_from_obj(among_us);
                model.set_scale([1.5;3]);
                model.set_pos([0.0, -0.5, 0.0]);
                model.set_pipeline_id(PIPELINE_UI_IMAGE_3D);

                model.prepare_uniforms(&engine.pipelines);

                *model.get_uniform(ShaderStage::Fragment, 0) = ShaderItem::Sampler2D(among_us_texture);

                let cuber = Drawable::cube_from_transform([4.0, 0.0, 0.0], 1.0, 1.0, 1.0, [1.0, 0.0, 0.0, 1.0]);
                let cubeg = Drawable::cube_from_transform([00.0, 4.0, 0.0], 1.0, 1.0, 1.0, [0.0, 1.0, 0.0, 1.0]);
                let cubeb = Drawable::cube_from_transform([00.0, 0.0, 4.0], 1.0, 1.0, 1.0, [0.0, 0.0, 1.0, 1.0]);

                engine.add_drawable(rect);
                engine.add_drawable(cuber);
                engine.add_drawable(cubeg);
                engine.add_drawable(cubeb);
                engine.add_drawable(model);

                let state = State {yaw: -270.0, pitch: 0.0, momentum: [0f32;3]};

                engine.world_view.set_target_yaw_pitch(state.yaw, state.pitch);

                return state;
            },

            |engine: &mut crate::engine::Engine, state, delta| {
            let speed = 5.0;

            for event in &engine.events {
                match event {
                    WindowEvent::KeyUp(keycode) => {
                        state.momentum[0] -= match *keycode {38 => -speed, 40 => speed, _ => 0.0};
                        state.momentum[2] -= match *keycode {25 => speed, 39 => -speed, _ => 0.0};
                        state.momentum[1] -= match *keycode {65 => speed, 37 => -speed, _ => 0.0};
                    }
                    WindowEvent::KeyDown(keycode, holding) => {
                        match *keycode {
                            36 => {panic!()},
                           _ => {println!("{}", *keycode)}
                        };

                        if !(*holding) {
                            state.momentum[0] += match *keycode {38 => -speed, 40 => speed, _ => 0.0};
                            state.momentum[2] += match *keycode {25 => speed, 39 => -speed, _ => 0.0};
                            state.momentum[1] += match *keycode {65 => speed, 37 => -speed, _ => 0.0};
                        };
                    },
                    WindowEvent::MouseMove(dx, dy) => {
                        if (*dx, *dy) != (0.0, 0.0) {
                            let sensitivity = 0.1;

                            state.yaw += -dx * sensitivity;
                            state.pitch += (dy * sensitivity).min(89.0).max(-89.0);

                            engine.world_view.set_target_yaw_pitch(state.yaw, state.pitch);

                            engine.window.set_mouse((engine.dimensions[0]/2) as f32, (engine.dimensions[1]/2) as f32);
                       }
                    },
                    _ => {}
                }

           }

            let momentum = state.momentum.iter().map(|&x| x * delta).collect::<Vec<f32>>().try_into().unwrap();

            engine.world_view.move_eye_local(momentum);
        })
    }
}
