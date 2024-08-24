use crate::engine::drawables::Drawable;

use crate::vulkan::image::Texture;
use crate::vulkan::pipeline::{BuiltinUniform, GraphicsPipeline, ShaderItem, ShaderStage, ShaderType, UniformType};
use crate::vulkan::uniform::DescriptorBindings;


impl Drawable { pub fn prepare_uniforms(&mut self, pipelines: &Vec<GraphicsPipeline>) {
    self.uniforms = std::collections::HashMap::new();

    let pipeline = &pipelines[self.get_pipeline_id()];

    for (shader_stage, uniforms) in pipeline.uniform_layout.iter() {
        self.uniforms.insert(shader_stage.clone(), uniforms.iter().map(|uniform_type| uniform_type.to_shader_item()).collect());
    }
}}

impl Drawable {pub fn get_uniform(&mut self, stage: ShaderStage, i: usize) -> &mut ShaderItem {
    return &mut self.uniforms.get_mut(&stage).unwrap()[i]
}}

impl crate::engine::Engine { pub fn add_drawable<'a>(&'a mut self, mut drawable: Drawable) -> &'a mut Drawable {
    if !drawable.uniforms.contains_key(&ShaderStage::Fragment) {
        drawable.prepare_uniforms(&self.pipelines);
    }

    drawable.update_vertice_coords();

    let pipeline_id = drawable.get_pipeline_id();

    self.pipelines[pipeline_id].uses += 1;

    let (uniform_layout, descriptor_bindings) = {
        let pipeline = &self.pipelines[pipeline_id];

        (
            pipeline.uniform_layout.clone(),
            pipeline.descriptor_bindings.clone()
         )
    };

    let descriptor_set_layout = self.pipeline_layouts.get(&descriptor_bindings).expect("drawable using non-initialized pipeline").1;

    let mut bindings = vec!();

    drawable.descriptor_sets = self.create_descriptor_sets(self.swapchain_images.len(), descriptor_set_layout);

    let empty = vec!();

    for (_, uniforms) in uniform_layout { for uniform in uniforms {
        let uniform_buffers = match uniform {
            UniformType::Builtin(ref builtin) => match builtin {
                BuiltinUniform::Camera3d => self.world_view.get_3d_uniform_buffers(),
                BuiltinUniform::Camera2d => self.world_view.get_2d_uniform_buffers(),
                _ => {
                    let (uniform_buffers, uniform_memories) = self.create_uniform_buffers(uniform.size_of());

                    let uniform_buffers = uniform_buffers.iter()
                        .zip(uniform_memories.iter())
                        .map(|(x, y)| (*x, *y))
                        .collect::<Vec<(_, _)>>();

                    drawable.uniform_buffers.push(uniform_buffers);
                    &drawable.uniform_buffers[drawable.uniform_buffers.len()-1]
                }
            },

            UniformType::Local(ref shader_type) => match shader_type {
                ShaderType::Sampler2D => &empty,
            },

            _ => {&empty}
        }.iter().map(|(buf, _mem)| return *buf).collect::<Vec<_>>();

        bindings.push(((bindings.len()) as u32, uniform_buffers, uniform.size_of()));
    }}

    let flat_uniforms = [
        drawable.uniforms.get(&ShaderStage::Vertex).unwrap().clone(),
        drawable.uniforms.get(&ShaderStage::Fragment).unwrap().clone()
    ].into_iter().flatten().collect();

    self.update_descriptor_sets(&drawable.descriptor_sets, bindings, flat_uniforms);

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
