```rust
fn main() {
    let vulkan_app = forsith::VulkanApp::new("forsith-plan", forsith::vulkan_version(0, 1, 0), 1)


    let render_target = forsith::native_window();

    let renderer = forsith::renderer::new(&mut vulkan_app, render_target);


    let uniform_layout = [68, 64]; // Either<sizes, SpecialUniform enum>

    let camera_uniform = forsith::UniformObject::new(&vulkan_app, ([[0f32;4];4], f32));

    let pipeline = forsith::pipeline::new(
        &mut renderer,
        "../shader.frag",
        "../shader.vert",
        uniform_layout,
        ([[0f32;4]])
    );


    let drawable_uniform = forsith::UniformObject::new(&vulkan_app, [[0f32;4];4]);

    let obj = forsith::drawable(&mut renderer, pipeline, drawable_uniform);


    while renderer.loop() {}
}
```
