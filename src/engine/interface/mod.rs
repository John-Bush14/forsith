pub mod drawables;


use drawables::{
    drawable
};


impl crate::engine::Engine { pub fn add_drawable<'a>(&'a mut self, mut drawable: drawable) -> &'a mut drawable {
    for vertex in drawable.get_vertices().clone() {
        if !self.vertex_indices.contains_key(&vertex) {
            self.vertex_usage_counts.insert(vertex, 0);
            
            self.vertices.push(vertex);

            self.vertex_indices.insert(vertex, (self.vertices.len()-1) as u16);
        }
        
        *self.vertex_usage_counts.get_mut(&vertex).unwrap() += 1;

        self.indices.push(*self.vertex_indices.get(&vertex).unwrap());

        drawable.indices.push(self.indices.len()-1);
    }
    
    drawable.id = self.drawables.len() + 1;

    let id = self.drawables.len();

    self.drawables.push(drawable);

    return &mut self.drawables[id];
}}

impl crate::engine::Engine { pub fn remove_drawable(&mut self, drawable_index: usize) {
    let drawable = self.drawables.remove(drawable_index);

    for i in 0..drawable.get_vertices().len() {
        let vertex = drawable.get_vertices()[i];
        

        let indice_index = drawable.indices[i];

        self.indices.remove(indice_index);


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
            
            for drawable in &mut self.drawables {for indice_t in &mut drawable.indices {if *indice_t >= indice as usize {*indice_t -= 1}}}
        }
    }
}}
