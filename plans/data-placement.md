```rust
struct VulkanApp {
    //instance
    //device with only graphics queue
    //transient command pool
    //graphics queue family
    //graphics queue
    //physical device
}

struct RenderTarget {
    //surface_khr
}

struct UniformObject {
    //buffer
    //uniforms
}

struct Renderer {
    //swapchains
    //drawables
    //command buffers + command pool
    //device with both queues
    //presentation / graphics queue
    //presentation queue family
    //pipelines
}

struct Pipeline {
    //uniform layout
}

struct Drawable {
    //vertices / buffer,
    //indices / buffer,
    //pipeline (id not Pipeline)
    //uniformObjects for pipeline
}
```