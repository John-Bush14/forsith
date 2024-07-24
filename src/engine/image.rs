impl super::Engine {pub fn create_texture_image(&self, file: String) {
    let image = image::open(file).expect("error getting image");
    let image_as_rgb = image.to_rgba();
    let image_width = (&image_as_rgb).width();
    let image_height = (&image_as_rgb).height();
    let pixels = image_as_rgb.into_raw();
    let image_size = (pixels.len() * std::mem::size_of::<u8>()) as u64;
}}

impl super::Engine {pub fn create_image(&mut self, file: String) {
    let image_texture = self.create_texture_image(file);
}}
