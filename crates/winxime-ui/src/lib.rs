use winxime_core::InputContext;

pub struct CandidateWindow {
    visible: bool,
}

impl CandidateWindow {
    pub fn new() -> Self {
        Self { visible: false }
    }
    
    pub fn show(&mut self) {
        self.visible = true;
    }
    
    pub fn hide(&mut self) {
        self.visible = false;
    }
    
    pub fn update(&mut self, _context: &InputContext) {
    }
    
    pub fn render(&self) {
        if self.visible {
        }
    }
}

impl Default for CandidateWindow {
    fn default() -> Self {
        Self::new()
    }
}