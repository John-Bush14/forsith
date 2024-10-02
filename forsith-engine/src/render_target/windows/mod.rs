pub trait Window {
    fn initialize_connection() -> Self where Self: Sized;

    fn map_window(&mut self);
}


pub fn create_native_window() -> Box<dyn Window> {todo!();}
