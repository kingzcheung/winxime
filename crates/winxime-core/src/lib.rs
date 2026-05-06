pub struct InputContext {
    pub composition: String,
    pub candidates: Vec<String>,
    pub selected_index: usize,
}

impl InputContext {
    pub fn new() -> Self {
        Self {
            composition: String::new(),
            candidates: Vec::new(),
            selected_index: 0,
        }
    }
}

impl Default for InputContext {
    fn default() -> Self {
        Self::new()
    }
}