## VulkanApp [***required for everything***]
### data
 * vulkan instance
 * device  
    \+ graphics queues (families)  
    \+ presentation queues (families)
    \+ physical device
 * transient command pool

### responsibilities
 * creating vulkan instance
 * choosing physical device with queue family with graphics queue + presentation queue for every renderer
 * creating device from this physical device with queues
 * creating transient command pool

### interface
##### pub(crate)
 * one time command buffers
 * device creation
 * getters(all)
 * claim queue pair (for renderer)

##### pub
 * ::new(count renderers)


## Pipeline [***data needed to create VkPipeline***]
### data
 * input layout
    * vertex shader inputs
    * uniforms for vertex and fragment shader
 * fragment and vertex shader module
 * Option\<VkPipeline\>

### responsibilities
 * creating fragment and vertex shader module
 * (re)creating and destroying VkPipeline

### interface
##### pub(crate)
 * getter(VkPipeline)

##### pub
 * ::new()


## UniformObject [***All data needed for a uniform***]
### data
 * actual uniform data
 * optional buffer
 * bindings
 * changed?

### responsibilities
 * creating bindings
 * updating bindings 

### interface
##### pub(crate)
 * getter(bindings)
 * poll_changed()

##### pub
 * get/setters(actual uniform data)



## RenderTarget [***can be rendered to***]
### data
 * surface khr
 * swapchain
 * images

### responsibilities
 * creating surface khr
 * creating swapchain
 * creating color, depth, ... images

### interface
##### pub(crate)
 * ::new()
 * getters(all)


## Drawable [***struct which can be drawn***]
### data
 * Vec\<UniformObject\>
 * vertices
 * indices
 * pipeline (index of renderer.pipelines)

### responsibilities
 * creating vertice + indice buffers

### interface
##### pub(crate)
 * poll_changed()
 * getters(uniformobjects, pipelines, buffers)

##### pub
 * set/getters(all)

## Renderer [***handles rendering drawables to a render target***]
### data
 * drawables
 * pipelines
 * command pool + buffers
 * render target

### responsibilies
 * record and enter command buffers with renderpass

### interface
##### pub
 * loop()
 * get/setters(drawables, pipelines, render target)
