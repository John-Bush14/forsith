use crate::engine::drawables::Drawable;

use crate::vulkan::image::Texture;
use crate::vulkan::pipeline::Uniform;
use crate::vulkan::uniform::DescriptorBindings;


impl crate::engine::Engine { pub fn add_drawable<'a>(&'a mut self, mut drawable: Drawable) -> &'a mut Drawable {
    drawable.update_vertice_coords();

    let pipeline_id = drawable.get_pipeline_id();

    self.pipelines[pipeline_id].uses += 1;

    let (descriptor_count, vertex_uniforms, fragment_uniforms) = {
        let pipeline = &self.pipelines[pipeline_id];

        (
            DescriptorBindings::from(&pipeline.vertex_uniforms, &pipeline.fragment_uniforms),
            pipeline.vertex_uniforms.clone(), pipeline.fragment_uniforms.clone()
        )
    };

    let descriptor_set_layout = self.pipeline_layouts.get(&descriptor_count).expect("drawable using non-initialized pipeline").1;

    let mut bindings = vec!();

    drawable.descriptor_sets = self.create_descriptor_sets(self.swapchain_images.len(), descriptor_set_layout);

    let empty = vec!();

    for uniform in vertex_uniforms.iter().chain(fragment_uniforms.iter()) {
        let uniform_buffers = match uniform {
            Uniform::Camera3d => self.world_view.get_3d_uniform_buffers(),
            Uniform::Camera2d => self.world_view.get_2d_uniform_buffers(),
            Uniform::Image => &empty,
            _ => {
                let (uniform_buffers, uniform_memories) = self.create_uniform_buffers(uniform.size_of());

                let uniform_buffers = uniform_buffers.iter()
                    .zip(uniform_memories.iter())
                    .map(|(x, y)| (*x, *y))
                    .collect::<Vec<(_, _)>>();

                drawable.uniform_buffers.push(uniform_buffers);
                &drawable.uniform_buffers[drawable.uniform_buffers.len()-1]
            },
        }.iter().map(|(buf, _mem)| return *buf).collect::<Vec<_>>();

        bindings.push(((bindings.len()) as u32, uniform_buffers, uniform.size_of()));
    }

    let default_image = Default::default();

    self.update_descriptor_sets(&drawable.descriptor_sets, bindings,
        if let Some(image) = &drawable.image {image}
        else {&default_image}
    );

    drawable.device = self.device;

    let mut indices = vec!();

    for &vertex in drawable.get_vertices() {
        if !self.vertex_indices.contains_key(&vertex) {
            self.vertex_usage_counts.insert(vertex, 0);

            self.vertices.push(vertex);

            self.vertex_indices.insert(vertex, (self.vertices.len()-1) as u16);
        }

        *self.vertex_usage_counts.get_mut(&vertex).unwrap() += 1;

        indices.push(*self.vertex_indices.get(&vertex).unwrap());
    }

    drawable.indices = indices;

    if drawable.indices.len() > 0 {
        (drawable.indice_buffer, drawable.indice_memory)
            = self.create_device_local_buffer_with_data::<u16, _>(0x00000040, &drawable.indices);
    }

    drawable.id = self.drawables.len() + 1;

    let id = self.drawables.len();

    self.drawables.push(drawable);

    return &mut self.drawables[id];
}}

impl crate::engine::Engine { pub fn remove_drawable(&mut self, drawable_index: usize) {
    let drawable = self.drawables.remove(drawable_index);

    let pipeline = &self.pipelines[drawable.get_pipeline_id()];

    let pipeline_key = self.pipelines.iter().position(|pipeline_| pipeline_.pipeline == pipeline.pipeline).unwrap();

    self.pipelines[pipeline_key].uses -= 1;

    for i in 0..drawable.get_vertices().len() {
        let vertex = drawable.get_vertices()[i];


        let usage_count = self.vertex_usage_counts.get_mut(&vertex).unwrap();

        *usage_count -= 1;


        if *usage_count <= 0 {
            let indice = *self.vertex_indices.get(&vertex).unwrap();

            self.vertices.remove(indice as usize);
            self.vertex_indices.remove(&vertex);

            for vertex in &self.vertices {
                let vertex_indice = self.vertex_indices.get_mut(&vertex).unwrap();

                if *vertex_indice >= indice {*vertex_indice -= 1;}
            }

            for drawable in &mut self.drawables {for indice_t in &mut drawable.indices {if *indice_t >= indice as u16 {*indice_t -= 1}}}
        }
    }
}}