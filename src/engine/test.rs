#[cfg(test)]
mod tests {
    use crate::engine;

    fn event_loop(engine: &mut crate::engine::Engine) {}

    #[test]
    fn template() {
        engine::initialize_engine("test".to_string(), [0, 0, 0], event_loop)
    }
}
