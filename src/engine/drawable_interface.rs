use crate::engine::drawables::Drawable;

use crate::vulkan::pipeline::Uniform;


impl crate::engine::Engine { pub fn add_drawable<'a>(&'a mut self, mut drawable: Drawable) -> &'a mut Drawable {
    let pipeline = self.pipelines[drawable.get_pipeline_id()].clone();
    
    let descriptor_count = pipeline.uniforms.len() as u32;
    
    let pipeline_key = self.pipelines.iter().position(|pipeline_| pipeline_.pipeline == pipeline.pipeline).unwrap();

    self.pipelines[pipeline_key].uses += 1;

    let descriptor_set_layout = self.pipeline_layouts.get(&descriptor_count).expect("drawable using non-initialized pipeline").1;

    let mut bindings = vec!();
        
    drawable.descriptor_sets = self.create_descriptor_sets(self.swapchain_images.len(), descriptor_set_layout);

    for uniform in &pipeline.uniforms {                           
        let uniform_buffers = match uniform {
            Uniform::Camera3d => self.world_view.get_3d_uniform_buffers().clone(),
            Uniform::Camera2d => self.world_view.get_2d_uniform_buffers().clone(),
            _ => {
                let (uniform_buffers, uniform_memories) = self.create_uniform_buffers(uniform.size_of());
    
                let uniform_buffers = uniform_buffers.iter()
                    .zip(uniform_memories.iter())
                    .map(|(x, y)| (*x, *y))
                    .collect::<Vec<(_, _)>>();

                drawable.uniform_buffers.push(uniform_buffers.clone());
                uniform_buffers
            },
        }.iter().map(|(buf, _mem)| return *buf).collect::<Vec<_>>();

        bindings.push(((bindings.len()) as u32, uniform_buffers, uniform.size_of()));
    }
    
    self.update_descriptor_sets(drawable.descriptor_sets.clone(), bindings);

    drawable.device = self.device;

    for vertex in drawable.get_vertices().clone() {
        drawable.vertices_changed.0 = true;

        if !self.vertex_indices.contains_key(&vertex) {
            self.vertex_usage_counts.insert(vertex, 0);
        
            self.vertices.push(vertex);

            drawable.vertices_changed.1 = true;

            self.vertex_indices.insert(vertex, (self.vertices.len()-1) as u16);
        }
    
        *self.vertex_usage_counts.get_mut(&vertex).unwrap() += 1;

        drawable.indices.push(*self.vertex_indices.get(&vertex).unwrap());
    }

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