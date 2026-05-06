mod text_input_processor;
mod class_factory;
mod dll;

pub use text_input_processor::TextInputProcessor;

pub struct TsfManager;

impl TsfManager {
    pub fn new() -> Self {
        Self
    }

    pub fn register() -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

impl Default for TsfManager {
    fn default() -> Self {
        Self::new()
    }
}